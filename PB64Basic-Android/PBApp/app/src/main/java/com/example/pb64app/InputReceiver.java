package com.example.pb64app;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;

/**
 * Test/automation hook for the console demos.  It feeds the same byte queue
 * used by WAITKEY$, INKEY$, and CON.LINE.INPUT without changing PB semantics.
 */
public final class InputReceiver extends BroadcastReceiver {
    public static final String ACTION = "com.example.pb64app.PUSH_INPUT";

    @Override
    public void onReceive(Context context, Intent intent) {
        if (intent == null || !ACTION.equals(intent.getAction())) {
            return;
        }
        String text = intent.getStringExtra("text");
        boolean newline = intent.getBooleanExtra("newline", false);
        MainActivity.pushInput(text == null ? "" : text, newline);
    }
}
