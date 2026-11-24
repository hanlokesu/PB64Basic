package com.example.pb64app;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.DialogInterface;
import android.media.AudioAttributes;
import android.media.AudioFormat;
import android.media.AudioTrack;
import android.media.Ringtone;
import android.media.RingtoneManager;
import android.net.Uri;
import android.os.Build;
import android.os.Handler;
import android.os.Looper;

import java.util.concurrent.CountDownLatch;

/**
 * Android implementations of the Win32 calls the PB runtime cannot satisfy by
 * itself: MessageBoxA (blocking, returns the Win32 ID* code) and the sound
 * primitives used by BEEP / PLAY SOUND.
 */
@SuppressWarnings("deprecation")
public final class PbAndroidBridge {
    private static final int IDOK = 1;
    private static final int IDCANCEL = 2;
    private static final int IDABORT = 3;
    private static final int IDRETRY = 4;
    private static final int IDIGNORE = 5;
    private static final int IDYES = 6;
    private static final int IDNO = 7;

    private PbAndroidBridge() {
    }

    /**
     * Blocks the calling native thread until the dialog is answered, matching
     * MessageBoxA.  Returns -1 when there is no live Activity, so the runtime
     * can fall back to its console representation instead of hanging.
     */
    public static int showMessageBox(String title, String text, int style) {
        MainActivity activity = MainActivity.getInstance();
        if (activity == null || activity.isFinishing()
                || Build.VERSION.SDK_INT >= 17 && activity.isDestroyed()) {
            return -1;
        }

        final CountDownLatch done = new CountDownLatch(1);
        final int[] result = {defaultResult(style)};
        final Handler handler = new Handler(Looper.getMainLooper());

        activity.runOnUiThread(() -> {
            AlertDialog.Builder builder = new AlertDialog.Builder(activity)
                    .setTitle(title == null || title.isEmpty() ? "PB64Basic" : title)
                    .setMessage(text == null ? "" : text)
                    .setCancelable(true);

            int kind = style & 0x0F;
            switch (kind) {
                case 0x01: // MB_OKCANCEL
                    builder.setPositiveButton("OK", (d, w) -> result[0] = IDOK);
                    builder.setNegativeButton("Cancel", (d, w) -> result[0] = IDCANCEL);
                    break;
                case 0x02: // MB_ABORTRETRYIGNORE
                    builder.setPositiveButton("Abort", (d, w) -> result[0] = IDABORT);
                    builder.setNeutralButton("Retry", (d, w) -> result[0] = IDRETRY);
                    builder.setNegativeButton("Ignore", (d, w) -> result[0] = IDIGNORE);
                    break;
                case 0x03: // MB_YESNOCANCEL
                    builder.setPositiveButton("Yes", (d, w) -> result[0] = IDYES);
                    builder.setNeutralButton("No", (d, w) -> result[0] = IDNO);
                    builder.setNegativeButton("Cancel", (d, w) -> result[0] = IDCANCEL);
                    break;
                case 0x04: // MB_YESNO
                    builder.setPositiveButton("Yes", (d, w) -> result[0] = IDYES);
                    builder.setNegativeButton("No", (d, w) -> result[0] = IDNO);
                    break;
                case 0x05: // MB_RETRYCANCEL
                    builder.setPositiveButton("Retry", (d, w) -> result[0] = IDRETRY);
                    builder.setNegativeButton("Cancel", (d, w) -> result[0] = IDCANCEL);
                    break;
                default: // MB_OK
                    builder.setPositiveButton("OK", (d, w) -> result[0] = IDOK);
                    break;
            }

            AlertDialog dialog = builder.create();
            dialog.setCanceledOnTouchOutside(false);
            dialog.setOnCancelListener(d -> result[0] = IDCANCEL);
            dialog.setOnDismissListener(d -> done.countDown());
            activity.setActiveDialog(dialog);
            dialog.show();

            int autoOkMs = activity.getAutoOkMs();
            if (autoOkMs > 0) {
                handler.postDelayed(() -> {
                    if (dialog.isShowing()) {
                        dialog.getButton(DialogInterface.BUTTON_POSITIVE).performClick();
                    }
                }, autoOkMs);
            }
        });

        try {
            done.await();
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return IDCANCEL;
        }
        activity.clearActiveDialog();
        return result[0];
    }

    private static int defaultResult(int style) {
        int kind = style & 0x0F;
        if (kind == 0x03 || kind == 0x04) {
            return IDYES;
        }
        return IDOK;
    }

    /** PLAY SOUND freq&, duration&: a blocking sine tone, like Win32 Beep(). */
    public static int playTone(int freq, int durationMs) {
        if (freq < 37 || freq > 32767 || durationMs < 0) {
            return -1;
        }
        if (durationMs == 0) {
            return 0;
        }
        int sampleRate = 44100;
        int samples = (int) Math.max(1L, (long) sampleRate * durationMs / 1000L);
        short[] pcm = new short[samples];
        double angular = 2.0 * Math.PI * freq / sampleRate;
        for (int i = 0; i < samples; i++) {
            pcm[i] = (short) (Short.MAX_VALUE * 0.65 * Math.sin(angular * i));
        }

        AudioTrack track = null;
        try {
            int minBytes = AudioTrack.getMinBufferSize(
                    sampleRate,
                    AudioFormat.CHANNEL_OUT_MONO,
                    AudioFormat.ENCODING_PCM_16BIT);
            int bytes = Math.max(pcm.length * 2, Math.max(minBytes, 1));
            if (Build.VERSION.SDK_INT >= 23) {
                track = new AudioTrack.Builder()
                        .setAudioAttributes(new AudioAttributes.Builder()
                                .setUsage(AudioAttributes.USAGE_MEDIA)
                                .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
                                .build())
                        .setAudioFormat(new AudioFormat.Builder()
                                .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
                                .setSampleRate(sampleRate)
                                .setChannelMask(AudioFormat.CHANNEL_OUT_MONO)
                                .build())
                        .setBufferSizeInBytes(bytes)
                        .setTransferMode(AudioTrack.MODE_STREAM)
                        .build();
            } else {
                track = new AudioTrack(
                        android.media.AudioManager.STREAM_MUSIC,
                        sampleRate,
                        AudioFormat.CHANNEL_OUT_MONO,
                        AudioFormat.ENCODING_PCM_16BIT,
                        bytes,
                        AudioTrack.MODE_STREAM);
            }
            track.play();
            track.write(pcm, 0, pcm.length);
            Thread.sleep(durationMs);
            return 0;
        } catch (Throwable t) {
            return -1;
        } finally {
            if (track != null) {
                try {
                    track.stop();
                } catch (Throwable ignored) {
                    // stop() throws when the track never reached PLAYSTATE_PLAYING.
                }
                track.release();
            }
        }
    }

    /** BEEP: the default notification sound, played asynchronously like MessageBeep. */
    public static int messageBeep() {
        MainActivity activity = MainActivity.getInstance();
        if (activity == null) {
            return -1;
        }
        try {
            Uri uri = RingtoneManager.getDefaultUri(RingtoneManager.TYPE_NOTIFICATION);
            Ringtone ringtone = RingtoneManager.getRingtone(activity, uri);
            if (ringtone == null) {
                return -1;
            }
            ringtone.play();
            return 0;
        } catch (Throwable t) {
            return -1;
        }
    }
}
