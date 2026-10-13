package com.fragilesilver.slot;

import android.app.NativeActivity;
import android.content.Intent;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.graphics.Matrix;
import android.media.ExifInterface;
import android.net.Uri;
import android.view.View;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;

/**
 * The one piece of Java in slot: a NativeActivity that can ask the system for a picture.
 *
 * {@link #pickPicture(String)} opens the gallery. When a picture is chosen it is shrunk to at
 * most {@link #LIMIT} pixels on its long side (turned upright by its EXIF tag) and written to
 * {@code dest} as a JPEG; the native side watches for that file. A cancelled or failed pick
 * writes {@code dest + ".cancel"} instead, so the native side never waits for ever.
 */
public class SlotActivity extends NativeActivity {
    private static final int PICK = 7001;
    private static final int LIMIT = 2048;
    private String dest;

    /**
     * Keeps the navigation and status bars out of the way, so the on-screen buttons are not
     * under the system's three-button bar (they come back for a moment on an edge swipe).
     */
    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) {
            getWindow().getDecorView().setSystemUiVisibility(
                    View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                            | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                            | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                            | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                            | View.SYSTEM_UI_FLAG_FULLSCREEN
                            | View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY);
        }
    }

    public void pickPicture(final String to) {
        dest = to;
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                try {
                    Intent pick = new Intent(Intent.ACTION_GET_CONTENT);
                    pick.setType("image/*");
                    pick.addCategory(Intent.CATEGORY_OPENABLE);
                    startActivityForResult(pick, PICK);
                } catch (Exception e) {
                    cancel(to);
                }
            }
        });
    }

    @Override
    protected void onActivityResult(int request, int result, Intent data) {
        super.onActivityResult(request, result, data);
        if (request != PICK) {
            return;
        }
        final String to = dest;
        if (to == null) {
            return;
        }
        if (result != RESULT_OK || data == null || data.getData() == null) {
            cancel(to);
            return;
        }
        final Uri uri = data.getData();
        new Thread(new Runnable() {
            @Override
            public void run() {
                try {
                    save(uri, to);
                } catch (Throwable e) {
                    cancel(to);
                }
            }
        }).start();
    }

    private void save(Uri uri, String to) throws Exception {
        BitmapFactory.Options bounds = new BitmapFactory.Options();
        bounds.inJustDecodeBounds = true;
        try (InputStream in = getContentResolver().openInputStream(uri)) {
            BitmapFactory.decodeStream(in, null, bounds);
        }
        int sample = 1;
        while (Math.max(bounds.outWidth, bounds.outHeight) / (sample * 2) >= LIMIT) {
            sample *= 2;
        }
        BitmapFactory.Options opts = new BitmapFactory.Options();
        opts.inSampleSize = sample;
        Bitmap bitmap;
        try (InputStream in = getContentResolver().openInputStream(uri)) {
            bitmap = BitmapFactory.decodeStream(in, null, opts);
        }
        if (bitmap == null) {
            throw new Exception("not a picture");
        }
        int turn = 0;
        try (InputStream in = getContentResolver().openInputStream(uri)) {
            int tag = new ExifInterface(in).getAttributeInt(
                    ExifInterface.TAG_ORIENTATION, ExifInterface.ORIENTATION_NORMAL);
            if (tag == ExifInterface.ORIENTATION_ROTATE_90) {
                turn = 90;
            } else if (tag == ExifInterface.ORIENTATION_ROTATE_180) {
                turn = 180;
            } else if (tag == ExifInterface.ORIENTATION_ROTATE_270) {
                turn = 270;
            }
        } catch (Exception ignored) {
        }
        if (turn != 0) {
            Matrix m = new Matrix();
            m.postRotate(turn);
            bitmap = Bitmap.createBitmap(bitmap, 0, 0, bitmap.getWidth(), bitmap.getHeight(), m, true);
        }
        File part = new File(to + ".part");
        try (FileOutputStream out = new FileOutputStream(part)) {
            bitmap.compress(Bitmap.CompressFormat.JPEG, 92, out);
        }
        if (!part.renameTo(new File(to))) {
            throw new Exception("could not place the picture");
        }
    }

    private void cancel(String to) {
        try {
            new FileOutputStream(to + ".cancel").close();
        } catch (Exception ignored) {
        }
    }
}
