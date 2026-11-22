package com.rustcraft.minecraft;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.Context;
import android.graphics.Bitmap;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.graphics.Rect;
import android.graphics.RectF;
import android.media.AudioAttributes;
import android.media.AudioFormat;
import android.media.AudioManager;
import android.media.AudioTrack;
import android.os.Bundle;
import android.text.InputType;
import android.view.KeyEvent;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;
import android.view.WindowManager;
import android.widget.EditText;

public class MainActivity extends Activity {
    static {
        System.loadLibrary("minecraft");
    }

    public static native void nativeInit(String saveDir);
    public static native void nativeFrame(int[] outPixels);
    public static native void nativeAudio(short[] outPcm);
    public static native void nativeKey(int code, int down);
    public static native void nativeChar(int code);
    public static native void nativeMouseMove(int dx, int dy);
    public static native void nativeMouseTo(int x, int y);
    public static native void nativeMouseButton(int button, int down);
    public static native void nativeWheel(int delta);
    public static native void nativeBlur();
    public static native int nativeUiState();

    private GameView gameView;
    private volatile boolean audioRunning = false;
    private Thread audioThread;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        nativeInit(getFilesDir().getAbsolutePath());
        gameView = new GameView(this);
        setContentView(gameView);
        hideSystemUi();
    }

    private void hideSystemUi() {
        getWindow().getDecorView().setSystemUiVisibility(
            View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
            | View.SYSTEM_UI_FLAG_FULLSCREEN
            | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
            | View.SYSTEM_UI_FLAG_LAYOUT_STABLE
            | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
            | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
        );
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) hideSystemUi();
    }

    @Override
    protected void onResume() {
        super.onResume();
        hideSystemUi();
        startAudio();
        gameView.resumeGame();
    }

    @Override
    protected void onPause() {
        super.onPause();
        nativeBlur();
        gameView.pauseGame();
        stopAudio();
    }

    private void startAudio() {
        if (audioRunning) return;
        audioRunning = true;
        audioThread = new Thread(() -> {
            int rate = 22050;
            int minBuf = AudioTrack.getMinBufferSize(
                rate, AudioFormat.CHANNEL_OUT_MONO, AudioFormat.ENCODING_PCM_16BIT
            );
            int bufBytes = Math.max(minBuf, 2048);
            AudioTrack track = new AudioTrack(
                new AudioAttributes.Builder()
                    .setUsage(AudioAttributes.USAGE_GAME)
                    .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
                    .build(),
                new AudioFormat.Builder()
                    .setSampleRate(rate)
                    .setChannelMask(AudioFormat.CHANNEL_OUT_MONO)
                    .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
                    .build(),
                bufBytes,
                AudioTrack.MODE_STREAM,
                AudioManager.AUDIO_SESSION_ID_GENERATE
            );
            short[] pcm = new short[512];
            try {
                track.play();
                while (audioRunning) {
                    nativeAudio(pcm);
                    track.write(pcm, 0, pcm.length);
                }
            } catch (Throwable ignored) {
            } finally {
                try { track.stop(); } catch (Throwable ignored) {}
                track.release();
            }
        }, "RustcraftAudio");
        audioThread.setDaemon(true);
        audioThread.start();
    }

    private void stopAudio() {
        audioRunning = false;
        if (audioThread != null) {
            try { audioThread.join(250); } catch (InterruptedException ignored) {}
            audioThread = null;
        }
    }

    public void openChatDialog() {
        runOnUiThread(() -> {
            EditText input = new EditText(this);
            input.setInputType(InputType.TYPE_CLASS_TEXT);
            input.setHint("Type chat or /command (e.g. /gamemode creative)");
            new AlertDialog.Builder(this)
                .setTitle("Chat / Command")
                .setView(input)
                .setPositiveButton("Send", (d, w) -> {
                    String txt = input.getText().toString();
                    nativeKey(0x54, 1);
                    nativeKey(0x54, 0);
                    for (int i = 0; i < txt.length(); i++) {
                        nativeChar(txt.charAt(i));
                    }
                    nativeKey(0x0D, 1);
                    nativeKey(0x0D, 0);
                    hideSystemUi();
                })
                .setNegativeButton("Cancel", (d, w) -> hideSystemUi())
                .setOnCancelListener(d -> hideSystemUi())
                .show();
        });
    }

    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        if (keyCode == KeyEvent.KEYCODE_BACK) {
            nativeKey(0x1B, 1);
            nativeKey(0x1B, 0);
            return true;
        }
        return super.onKeyDown(keyCode, event);
    }

    private static class GameView extends SurfaceView implements Runnable, SurfaceHolder.Callback {
        private static final int FB_W = 960;
        private static final int FB_H = 540;
        private static final int UI_W = 480;
        private static final int UI_H = 270;

        // Win32/Game VK constants
        private static final int VK_SHIFT = 0x10;
        private static final int VK_CTRL = 0x11;
        private static final int VK_ESC = 0x1B;
        private static final int VK_SPACE = 0x20;
        private static final int VK_A = 0x41;
        private static final int VK_D = 0x44;
        private static final int VK_E = 0x45;
        private static final int VK_S = 0x53;
        private static final int VK_W = 0x57;
        private static final int VK_Z = 0x5A;
        private static final int VK_F5 = 0x74;

        private final MainActivity activity;
        private final SurfaceHolder holder;
        private final int[] pixels = new int[FB_W * FB_H];
        private final Bitmap bitmap = Bitmap.createBitmap(FB_W, FB_H, Bitmap.Config.ARGB_8888);
        private final Rect srcRect = new Rect(0, 0, FB_W, FB_H);
        private final Rect dstRect = new Rect();
        private final Paint bmpPaint = new Paint();
        private final Paint uiFill = new Paint(Paint.ANTI_ALIAS_FLAG);
        private final Paint uiStroke = new Paint(Paint.ANTI_ALIAS_FLAG);
        private final Paint uiText = new Paint(Paint.ANTI_ALIAS_FLAG);

        private volatile boolean running = false;
        private Thread renderThread;
        private int viewW = 1, viewH = 1;
        private int uiState = 0;

        // Touch tracking
        private int stickPtr = -1;
        private float stickBaseX, stickBaseY, stickCurX, stickCurY;
        private boolean keyW, keyA, keyS, keyD, keySprint;
        private boolean sneakOn = false;
        private boolean zoomOn = false;

        private int lookPtr = -1;
        private float lookLastX, lookLastY;

        private int atkPtr = -1;
        private int usePtr = -1;
        private int jmpPtr = -1;

        // Touch button bounds in screen pixels
        private final RectF btnAtk = new RectF();
        private final RectF btnUse = new RectF();
        private final RectF btnJmp = new RectF();
        private final RectF btnSneak = new RectF();
        private final RectF btnInv = new RectF();
        private final RectF btnChat = new RectF();
        private final RectF btnCam = new RectF();
        private final RectF btnZoom = new RectF();
        private final RectF btnPause = new RectF();
        private final RectF stickZone = new RectF();

        GameView(MainActivity act) {
            super(act);
            this.activity = act;
            this.holder = getHolder();
            this.holder.addCallback(this);
            bmpPaint.setFilterBitmap(false);
            bmpPaint.setDither(false);
            uiStroke.setStyle(Paint.Style.STROKE);
            uiText.setColor(Color.WHITE);
            uiText.setTextAlign(Paint.Align.CENTER);
        }

        private void layoutRects(int w, int h) {
            viewW = Math.max(1, w);
            viewH = Math.max(1, h);
            float targetAspect = (float) FB_W / FB_H;
            float viewAspect = (float) viewW / viewH;
            int dw, dh, dx, dy;
            if (viewAspect > targetAspect) {
                dh = viewH;
                dw = Math.round(dh * targetAspect);
                dx = (viewW - dw) / 2;
                dy = 0;
            } else {
                dw = viewW;
                dh = Math.round(dw / targetAspect);
                dx = 0;
                dy = (viewH - dh) / 2;
            }
            dstRect.set(dx, dy, dx + dw, dy + dh);

            float unit = Math.min(viewW, viewH) / 10.0f;
            stickZone.set(0, viewH * 0.35f, viewW * 0.36f, viewH);
            stickBaseX = dx + unit * 2.1f;
            stickBaseY = viewH - unit * 2.3f;
            if (stickPtr == -1) {
                stickCurX = stickBaseX;
                stickCurY = stickBaseY;
            }
            btnSneak.set(dx + unit * 0.35f, viewH - unit * 1.15f, dx + unit * 1.45f, viewH - unit * 0.3f);

            float rx = dx + dw - unit * 0.4f;
            float by = viewH - unit * 0.35f;
            float bs = unit * 1.45f;
            btnJmp.set(rx - bs, by - bs, rx, by);
            btnUse.set(rx - bs * 2.25f, by - bs, rx - bs * 1.25f, by);
            btnAtk.set(rx - bs, by - bs * 2.25f, rx, by - bs * 1.25f);

            float ty = unit * 0.25f;
            float tw = unit * 1.15f;
            float th = unit * 0.72f;
            float gap = unit * 0.16f;
            float cx0 = viewW / 2.0f - (5 * tw + 4 * gap) / 2.0f;
            btnInv.set(cx0, ty, cx0 + tw, ty + th);
            btnChat.set(cx0 + (tw + gap), ty, cx0 + 2 * tw + gap, ty + th);
            btnCam.set(cx0 + 2 * (tw + gap), ty, cx0 + 3 * tw + 2 * gap, ty + th);
            btnZoom.set(cx0 + 3 * (tw + gap), ty, cx0 + 4 * tw + 3 * gap, ty + th);
            btnPause.set(cx0 + 4 * (tw + gap), ty, cx0 + 5 * tw + 4 * gap, ty + th);
            uiText.setTextSize(unit * 0.34f);
            uiStroke.setStrokeWidth(Math.max(2f, unit * 0.05f));
        }

        @Override
        public void surfaceCreated(SurfaceHolder h) {
            layoutRects(getWidth(), getHeight());
        }

        @Override
        public void surfaceChanged(SurfaceHolder h, int format, int w, int height) {
            layoutRects(w, height);
        }

        @Override
        public void surfaceDestroyed(SurfaceHolder h) {}

        void resumeGame() {
            if (running) return;
            running = true;
            renderThread = new Thread(this, "RustcraftRender");
            renderThread.start();
        }

        void pauseGame() {
            running = false;
            if (renderThread != null) {
                try { renderThread.join(400); } catch (InterruptedException ignored) {}
                renderThread = null;
            }
        }

        @Override
        public void run() {
            while (running) {
                long t0 = System.nanoTime();
                nativeFrame(pixels);
                uiState = nativeUiState();
                bitmap.setPixels(pixels, 0, FB_W, 0, 0, FB_W, FB_H);

                Canvas c = holder.lockCanvas();
                if (c != null) {
                    try {
                        c.drawColor(Color.BLACK);
                        c.drawBitmap(bitmap, srcRect, dstRect, bmpPaint);
                        if (uiState == 1) {
                            drawTouchHud(c);
                        }
                    } finally {
                        holder.unlockCanvasAndPost(c);
                    }
                }
                long elapsedMs = (System.nanoTime() - t0) / 1_000_000L;
                if (elapsedMs < 16) {
                    try { Thread.sleep(16 - elapsedMs); } catch (InterruptedException ignored) {}
                }
            }
        }

        private void drawBtn(Canvas c, RectF r, String label, boolean active) {
            uiFill.setColor(active ? Color.argb(150, 80, 175, 95) : Color.argb(95, 22, 24, 28));
            c.drawRoundRect(r, 12f, 12f, uiFill);
            uiStroke.setColor(active ? Color.argb(230, 170, 255, 185) : Color.argb(150, 210, 210, 220));
            c.drawRoundRect(r, 12f, 12f, uiStroke);
            float ty = r.centerY() - (uiText.descent() + uiText.ascent()) * 0.5f;
            c.drawText(label, r.centerX(), ty, uiText);
        }

        private void drawTouchHud(Canvas c) {
            float rad = Math.min(viewW, viewH) * 0.14f;
            uiFill.setColor(Color.argb(65, 20, 22, 26));
            c.drawCircle(stickBaseX, stickBaseY, rad, uiFill);
            uiStroke.setColor(Color.argb(140, 220, 220, 230));
            c.drawCircle(stickBaseX, stickBaseY, rad, uiStroke);

            uiFill.setColor(stickPtr != -1 ? Color.argb(180, 245, 245, 255) : Color.argb(130, 220, 220, 230));
            c.drawCircle(stickCurX, stickCurY, rad * 0.42f, uiFill);

            drawBtn(c, btnSneak, "SNK", sneakOn);
            drawBtn(c, btnAtk, "ATK", atkPtr != -1);
            drawBtn(c, btnUse, "USE", usePtr != -1);
            drawBtn(c, btnJmp, "JMP", jmpPtr != -1);

            drawBtn(c, btnInv, "INV", false);
            drawBtn(c, btnChat, "CHAT", false);
            drawBtn(c, btnCam, "CAM", false);
            drawBtn(c, btnZoom, "ZOOM", zoomOn);
            drawBtn(c, btnPause, "II", false);
        }

        private void setMoveKey(int code, boolean cur, boolean next) {
            if (cur != next) nativeKey(code, next ? 1 : 0);
        }

        private void updateStick(float x, float y) {
            float rad = Math.min(viewW, viewH) * 0.14f;
            float dx = x - stickBaseX;
            float dy = y - stickBaseY;
            float d = (float) Math.hypot(dx, dy);
            if (d > rad) {
                dx = dx / d * rad;
                dy = dy / d * rad;
            }
            stickCurX = stickBaseX + dx;
            stickCurY = stickBaseY + dy;
            float nx = dx / rad;
            float ny = dy / rad;

            boolean w = ny < -0.22f;
            boolean s = ny > 0.25f;
            boolean a = nx < -0.25f;
            boolean dKey = nx > 0.25f;
            boolean spr = ny < -0.72f;

            setMoveKey(VK_W, keyW, w); keyW = w;
            setMoveKey(VK_S, keyS, s); keyS = s;
            setMoveKey(VK_A, keyA, a); keyA = a;
            setMoveKey(VK_D, keyD, dKey); keyD = dKey;
            setMoveKey(VK_CTRL, keySprint, spr); keySprint = spr;
        }

        private void releaseStick() {
            stickPtr = -1;
            stickCurX = stickBaseX;
            stickCurY = stickBaseY;
            setMoveKey(VK_W, keyW, false); keyW = false;
            setMoveKey(VK_S, keyS, false); keyS = false;
            setMoveKey(VK_A, keyA, false); keyA = false;
            setMoveKey(VK_D, keyD, false); keyD = false;
            setMoveKey(VK_CTRL, keySprint, false); keySprint = false;
        }

        private int toFbX(float sx) {
            return Math.max(0, Math.min(UI_W - 1, Math.round((sx - dstRect.left) * UI_W / Math.max(1f, dstRect.width()))));
        }

        private int toFbY(float sy) {
            return Math.max(0, Math.min(UI_H - 1, Math.round((sy - dstRect.top) * UI_H / Math.max(1f, dstRect.height()))));
        }

        private boolean tryTapHotbar(float sx, float sy) {
            int fx = toFbX(sx);
            int fy = toFbY(sy);
            // Hotbar in render.rs: HOT_Y = 240, SLOT = 22, hotbar_x(i) = 480/2 - (9*24)/2 + i*24 = 132 + i*24
            if (fy >= 236 && fy <= 268) {
                for (int i = 0; i < 9; i++) {
                    int hx = 132 + i * 24;
                    if (fx >= hx - 1 && fx <= hx + 23) {
                        int code = 49 + i;
                        nativeKey(code, 1);
                        nativeKey(code, 0);
                        return true;
                    }
                }
            }
            return false;
        }

        @Override
        public boolean onTouchEvent(MotionEvent e) {
            int action = e.getActionMasked();
            int idx = e.getActionIndex();
            int pid = e.getPointerId(idx);
            float x = e.getX(idx);
            float y = e.getY(idx);

            if (uiState != 1) {
                int fx = toFbX(x);
                int fy = toFbY(y);
                if (action == MotionEvent.ACTION_DOWN || action == MotionEvent.ACTION_POINTER_DOWN) {
                    nativeMouseTo(fx, fy);
                    nativeMouseButton(0, 1);
                } else if (action == MotionEvent.ACTION_MOVE) {
                    nativeMouseTo(toFbX(e.getX(0)), toFbY(e.getY(0)));
                } else if (action == MotionEvent.ACTION_UP || action == MotionEvent.ACTION_POINTER_UP || action == MotionEvent.ACTION_CANCEL) {
                    nativeMouseTo(fx, fy);
                    nativeMouseButton(0, 0);
                }
                return true;
            }

            switch (action) {
                case MotionEvent.ACTION_DOWN:
                case MotionEvent.ACTION_POINTER_DOWN: {
                    if (btnPause.contains(x, y)) {
                        nativeKey(VK_ESC, 1); nativeKey(VK_ESC, 0);
                    } else if (btnInv.contains(x, y)) {
                        nativeKey(VK_E, 1); nativeKey(VK_E, 0);
                    } else if (btnChat.contains(x, y)) {
                        activity.openChatDialog();
                    } else if (btnCam.contains(x, y)) {
                        nativeKey(VK_F5, 1); nativeKey(VK_F5, 0);
                    } else if (btnZoom.contains(x, y)) {
                        zoomOn = !zoomOn;
                        nativeKey(VK_Z, zoomOn ? 1 : 0);
                    } else if (btnSneak.contains(x, y)) {
                        sneakOn = !sneakOn;
                        nativeKey(VK_SHIFT, sneakOn ? 1 : 0);
                    } else if (btnAtk.contains(x, y)) {
                        atkPtr = pid;
                        nativeMouseButton(0, 1);
                    } else if (btnUse.contains(x, y)) {
                        usePtr = pid;
                        nativeMouseButton(1, 1);
                    } else if (btnJmp.contains(x, y)) {
                        jmpPtr = pid;
                        nativeKey(VK_SPACE, 1);
                    } else if (tryTapHotbar(x, y)) {
                        // Hotbar slot switched
                    } else if (stickZone.contains(x, y) && stickPtr == -1) {
                        stickPtr = pid;
                        updateStick(x, y);
                    } else if (lookPtr == -1) {
                        lookPtr = pid;
                        lookLastX = x;
                        lookLastY = y;
                    }
                    break;
                }
                case MotionEvent.ACTION_MOVE: {
                    for (int i = 0; i < e.getPointerCount(); i++) {
                        int p = e.getPointerId(i);
                        float px = e.getX(i);
                        float py = e.getY(i);
                        if (p == stickPtr) {
                            updateStick(px, py);
                        } else if (p == lookPtr) {
                            float scale = 480f / Math.max(1f, dstRect.width());
                            int dx = Math.round((px - lookLastX) * scale * 1.35f);
                            int dy = Math.round((py - lookLastY) * scale * 1.35f);
                            if (dx != 0 || dy != 0) {
                                nativeMouseMove(dx, dy);
                                lookLastX = px;
                                lookLastY = py;
                            }
                        }
                    }
                    break;
                }
                case MotionEvent.ACTION_UP:
                case MotionEvent.ACTION_POINTER_UP:
                case MotionEvent.ACTION_CANCEL: {
                    if (pid == stickPtr) {
                        releaseStick();
                    } else if (pid == lookPtr) {
                        lookPtr = -1;
                    } else if (pid == atkPtr) {
                        atkPtr = -1;
                        nativeMouseButton(0, 0);
                    } else if (pid == usePtr) {
                        usePtr = -1;
                        nativeMouseButton(1, 0);
                    } else if (pid == jmpPtr) {
                        jmpPtr = -1;
                        nativeKey(VK_SPACE, 0);
                    }
                    break;
                }
            }
            return true;
        }
    }
}
