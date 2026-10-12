//! Where slot keeps its files on Android.
//!
//! With "All files access" granted, everything lives in a plain `slot/` folder on shared
//! storage (ROMs, saves, states, config), so a file manager or USB cable can reach it. Until
//! then slot uses its app-private folder, which works but is hidden from other apps.
use std::ffi::c_void;
use std::path::{Path, PathBuf};

use jni::objects::{JObject, JValue};
use jni::JavaVM;

const PACKAGE: &str = "com.fragilesilver.slot";
const SHARED: &str = "/storage/emulated/0/slot";
const ASKED: &str = ".asked_all_files_access";

pub fn shared_root() -> PathBuf {
    PathBuf::from(SHARED)
}

/// `Environment.isExternalStorageManager()` (Android 11+); false anywhere it is unavailable.
pub fn all_files_granted(vm: *mut c_void) -> bool {
    let Ok(vm) = (unsafe { JavaVM::from_raw(vm.cast()) }) else {
        return false;
    };
    let Ok(mut env) = vm.attach_current_thread() else {
        return false;
    };
    let result = env
        .call_static_method(
            "android/os/Environment",
            "isExternalStorageManager",
            "()Z",
            &[],
        )
        .and_then(|v| v.z());
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_clear();
        return false;
    }
    result.unwrap_or(false)
}

/// Opens the system's "Allow access to manage all files" page for this app.
pub fn ask_all_files(vm: *mut c_void, activity: *mut c_void) {
    let Ok(vm) = (unsafe { JavaVM::from_raw(vm.cast()) }) else {
        return;
    };
    let Ok(mut env) = vm.attach_current_thread() else {
        return;
    };
    let activity = unsafe { JObject::from_raw(activity as jni::sys::jobject) };
    let sent = (|| -> jni::errors::Result<()> {
        let action = env.new_string("android.settings.MANAGE_APP_ALL_FILES_ACCESS_PERMISSION")?;
        let intent = env.new_object(
            "android/content/Intent",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&action)],
        )?;
        let uri_text = env.new_string(format!("package:{PACKAGE}"))?;
        let uri = env
            .call_static_method(
                "android/net/Uri",
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&uri_text)],
            )?
            .l()?;
        env.call_method(
            &intent,
            "setData",
            "(Landroid/net/Uri;)Landroid/content/Intent;",
            &[JValue::Object(&uri)],
        )?;
        env.call_method(
            &activity,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[JValue::Object(&intent)],
        )?;
        Ok(())
    })();
    if sent.is_err() {
        let _ = env.exception_clear();
    }
}

/// Asks the gallery for a picture, to be written to `dest` (or `dest.cancel` when there is none).
/// When the activity has no picker (an old install), the cancel marker is written here so the
/// caller is never left waiting.
pub fn pick_picture(vm: *mut c_void, activity: *mut c_void, dest: &Path) {
    let cancel = || {
        let _ = std::fs::write(format!("{}.cancel", dest.display()), b"");
    };
    let Ok(vm) = (unsafe { JavaVM::from_raw(vm.cast()) }) else {
        return cancel();
    };
    let Ok(mut env) = vm.attach_current_thread() else {
        return cancel();
    };
    let activity = unsafe { JObject::from_raw(activity as jni::sys::jobject) };
    let sent = (|| -> jni::errors::Result<()> {
        let path = env.new_string(dest.display().to_string())?;
        env.call_method(
            &activity,
            "pickPicture",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&path)],
        )?;
        Ok(())
    })();
    if sent.is_err() {
        let _ = env.exception_clear();
        cancel();
    }
}

/// True once per install: the permission page is shown once, not on every launch.
pub fn first_ask(private: &Path) -> bool {
    let marker = private.join(ASKED);
    if marker.exists() {
        return false;
    }
    let _ = std::fs::write(marker, b"");
    true
}

/// Copies what the app-private folder holds into the shared folder, skipping anything that is
/// already there, so switching over keeps the library, saves and settings.
pub fn migrate(from: &Path, to: &Path) {
    fn copy(from: &Path, to: &Path) {
        let Ok(entries) = std::fs::read_dir(from) else {
            return;
        };
        let _ = std::fs::create_dir_all(to);
        for entry in entries.flatten() {
            let (src, dst) = (entry.path(), to.join(entry.file_name()));
            match entry.file_type() {
                Ok(t) if t.is_dir() => copy(&src, &dst),
                Ok(t) if t.is_file() && !dst.exists() => {
                    let _ = std::fs::copy(&src, &dst);
                }
                _ => {}
            }
        }
    }
    copy(from, to);
}

