package com.example.pb64app;

import android.app.Activity;
import android.app.AlertDialog;
import android.graphics.Color;
import android.graphics.Insets;
import android.graphics.Typeface;
import android.os.Build;
import android.os.Bundle;
import android.text.SpannableString;
import android.text.Spanned;
import android.text.style.BackgroundColorSpan;
import android.text.style.ForegroundColorSpan;
import android.view.Gravity;
import android.view.View;
import android.view.WindowInsets;
import android.view.inputmethod.EditorInfo;
import android.widget.Button;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;

@SuppressWarnings("deprecation")
public class MainActivity extends Activity {
    static {
        System.loadLibrary("pbapp");
    }

    private static MainActivity sInstance;

    private native String runPb();

    private native String runPbWithInput(String initial);

    private native short[] runPbAttrs();

    private native void pushText(String text, boolean newline);

    private native void pushKey(int key);

    private native int setWorkDir(String path);

    private TextView output;
    private EditText input;
    private Button runButton;
    private volatile boolean running;
    private volatile AlertDialog activeDialog;
    private String initialInput;
    private int autoOkMs;

    public static MainActivity getInstance() {
        return sInstance;
    }

    public static boolean pushInput(String text, boolean newline) {
        MainActivity activity = sInstance;
        if (activity == null) {
            return false;
        }
        activity.pushText(text, newline);
        return true;
    }

    public int getAutoOkMs() {
        return autoOkMs;
    }

    public void setActiveDialog(AlertDialog dialog) {
        activeDialog = dialog;
    }

    public void clearActiveDialog() {
        activeDialog = null;
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        sInstance = this;
        initialInput = getIntent().getStringExtra("pb64_input");
        autoOkMs = getIntent().getIntExtra("pb64_auto_ok_ms", 0);

        // PB file statements use relative paths; keep them inside the app's
        // private files directory instead of an unwritable process cwd.
        setWorkDir(getFilesDir().getAbsolutePath());

        final int pad = dp(24);
        final ScrollView scroll = new ScrollView(this);
        scroll.setFillViewport(false);

        final LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setBackgroundColor(Color.WHITE);
        root.setPadding(pad, pad, pad, pad);
        scroll.addView(root, new ScrollView.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.WRAP_CONTENT));

        // Android 15 (targetSdk 35) draws edge-to-edge by default.  Inset the
        // content ourselves so the title never hides under the status bar.
        scroll.setOnApplyWindowInsetsListener((v, insets) -> {
            int top;
            int bottom;
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                Insets bars = insets.getInsets(WindowInsets.Type.systemBars());
                top = bars.top;
                bottom = bars.bottom;
            } else {
                top = insets.getSystemWindowInsetTop();
                bottom = insets.getSystemWindowInsetBottom();
            }
            root.setPadding(pad, pad + top, pad, pad + bottom);
            return insets;
        });

        TextView title = new TextView(this);
        title.setText(getString(R.string.app_name));
        title.setTextSize(22);
        title.setTypeface(Typeface.DEFAULT_BOLD);
        title.setTextColor(Color.rgb(33, 33, 33));
        root.addView(title);

        TextView subtitle = new TextView(this);
        subtitle.setText("PowerBASIC native runtime \u00b7 console capture");
        subtitle.setTextSize(13);
        subtitle.setTextColor(Color.rgb(120, 120, 120));
        subtitle.setPadding(0, dp(4), 0, dp(12));
        root.addView(subtitle);

        output = new TextView(this);
        output.setText("Tap Run to execute the compiled program");
        output.setTextSize(14);
        output.setTypeface(Typeface.MONOSPACE);
        output.setTextColor(Color.rgb(224, 224, 224));
        output.setBackgroundColor(Color.BLACK);
        output.setPadding(dp(16), dp(16), dp(16), dp(16));
        output.setLineSpacing(0f, 1.15f);
        output.setGravity(Gravity.START);
        output.setTextIsSelectable(true);
        LinearLayout.LayoutParams outputLp = new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.WRAP_CONTENT);
        outputLp.setMargins(0, 0, 0, dp(16));
        root.addView(output, outputLp);

        LinearLayout inputRow = new LinearLayout(this);
        inputRow.setOrientation(LinearLayout.HORIZONTAL);
        inputRow.setGravity(Gravity.CENTER_VERTICAL);

        input = new EditText(this);
        input.setHint("Input for WAITKEY$ / INPUT");
        input.setTextSize(14);
        input.setSingleLine(true);
        input.setImeOptions(EditorInfo.IME_ACTION_SEND);
        input.setOnEditorActionListener((v, actionId, event) -> {
            if (actionId == EditorInfo.IME_ACTION_SEND) {
                sendLine();
                return true;
            }
            return false;
        });
        LinearLayout.LayoutParams inputLp = new LinearLayout.LayoutParams(
                0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f);
        inputLp.setMargins(0, 0, dp(8), 0);
        inputRow.addView(input, inputLp);

        Button sendLineButton = new Button(this);
        sendLineButton.setText("Line");
        sendLineButton.setAllCaps(false);
        sendLineButton.setOnClickListener(v -> sendLine());
        inputRow.addView(sendLineButton, new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.WRAP_CONTENT,
                LinearLayout.LayoutParams.WRAP_CONTENT));

        Button sendKeyButton = new Button(this);
        sendKeyButton.setText("Key");
        sendKeyButton.setAllCaps(false);
        sendKeyButton.setOnClickListener(v -> sendKey());
        inputRow.addView(sendKeyButton, new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.WRAP_CONTENT,
                LinearLayout.LayoutParams.WRAP_CONTENT));

        LinearLayout.LayoutParams inputRowLp = new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.WRAP_CONTENT);
        inputRowLp.setMargins(0, 0, 0, dp(12));
        root.addView(inputRow, inputRowLp);

        LinearLayout buttons = new LinearLayout(this);
        buttons.setOrientation(LinearLayout.HORIZONTAL);

        runButton = new Button(this);
        runButton.setText("Run Program");
        runButton.setAllCaps(false);
        runButton.setOnClickListener(v -> runInBackground());
        LinearLayout.LayoutParams runLp = new LinearLayout.LayoutParams(
                0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f);
        runLp.setMargins(0, 0, dp(8), 0);
        buttons.addView(runButton, runLp);

        Button clearButton = new Button(this);
        clearButton.setText("Clear");
        clearButton.setAllCaps(false);
        clearButton.setOnClickListener(v -> {
            if (!running) {
                output.setText("Tap Run to execute the compiled program");
            }
        });
        buttons.addView(clearButton, new LinearLayout.LayoutParams(
                0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f));
        root.addView(buttons);

        setContentView(scroll);
        scroll.requestApplyInsets();
        runButton.post(this::runInBackground);
    }

    @Override
    protected void onNewIntent(android.content.Intent intent) {
        super.onNewIntent(intent);
        setIntent(intent);
        initialInput = intent.getStringExtra("pb64_input");
        autoOkMs = intent.getIntExtra("pb64_auto_ok_ms", 0);
        runInBackground();
    }

    @Override
    protected void onDestroy() {
        if (sInstance == this) {
            sInstance = null;
        }
        AlertDialog dialog = activeDialog;
        activeDialog = null;
        if (dialog != null && dialog.isShowing()) {
            dialog.dismiss();
        }
        super.onDestroy();
    }

    private void sendLine() {
        String text = input.getText().toString();
        pushText(text, true);
        input.setText("");
    }

    private void sendKey() {
        String text = input.getText().toString();
        if (text.isEmpty()) {
            return;
        }
        pushKey((int)text.charAt(0));
        input.setText("");
    }

    private void runInBackground() {
        if (running) {
            return;
        }
        running = true;
        runButton.setEnabled(false);
        output.setText("Running...");

        new Thread(() -> {
            String text;
            short[] attrs;
            try {
                text = initialInput == null ? runPb() : runPbWithInput(initialInput);
                attrs = runPbAttrs();
            } catch (Throwable t) {
                text = "Native error: " + t;
                attrs = new short[0];
            }
            final String result = (text == null || text.trim().isEmpty())
                    ? "(no console output)" : text;
            final short[] resultAttrs = attrs == null ? new short[0] : attrs;
            runOnUiThread(() -> {
                output.setText(styled(result, resultAttrs));
                runButton.setEnabled(true);
                running = false;
            });
        }, "pb64basic-run").start();
    }

    /* Windows console attributes are two 4-bit palettes: low nibble is the
     * foreground, high nibble is the background.  Render them as real Android
     * spans so PB COLOR 15,1 is visible instead of being discarded. */
    private CharSequence styled(String text, short[] attrs) {
        SpannableString span = new SpannableString(text);
        if (attrs.length < text.length()) {
            return span;
        }
        for (int i = 0; i < text.length(); i++) {
            if (text.charAt(i) == '\n') {
                continue;
            }
            int attr = attrs[i] & 0xFFFF;
            int fg = attr & 0x0F;
            int bg = (attr >> 4) & 0x0F;
            span.setSpan(new ForegroundColorSpan(consoleColor(fg, true)),
                    i, i + 1, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE);
            span.setSpan(new BackgroundColorSpan(consoleColor(bg, false)),
                    i, i + 1, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE);
        }
        return span;
    }

    private static int consoleColor(int index, boolean foreground) {
        switch (index) {
            case 0: return 0xFF000000;
            case 1: return 0xFF000080;
            case 2: return 0xFF008000;
            case 3: return 0xFF008080;
            case 4: return 0xFF800000;
            case 5: return 0xFF800080;
            case 6: return 0xFF808000;
            case 7: return foreground ? 0xFFE0E0E0 : 0xFFC0C0C0;
            case 8: return 0xFF808080;
            case 9: return 0xFF0000FF;
            case 10: return 0xFF00FF00;
            case 11: return 0xFF00FFFF;
            case 12: return 0xFFFF0000;
            case 13: return 0xFFFF00FF;
            case 14: return 0xFFFFFF00;
            case 15: return 0xFFFFFFFF;
            default: return foreground ? 0xFFE0E0E0 : 0xFF111111;
        }
    }

    private int dp(int value) {
        return (int) (value * getResources().getDisplayMetrics().density + 0.5f);
    }
}
