//! `slot_power::Platform` on Android: battery via the sticky BATTERY_CHANGED broadcast
//! (apps cannot read sysfs), wall clock from the system, no LED/backlight/lid.
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use jni::objects::{JObject, JObjectArray, JValue};
use jni::JavaVM;
use slot_power::{Battery, Charge, LedState, Platform};

const REFRESH: Duration = Duration::from_secs(10);

pub struct AndroidPlatform {
    root: PathBuf,
    vm: JavaVM,
    activity: jni::sys::jobject,
    cache: Mutex<Option<(Instant, Option<Battery>)>>,
    phones: Mutex<Option<(Instant, bool)>>,
    rumble: (u16, Option<Instant>),
}

// The activity pointer is a JNI global reference owned by android-activity for the life of
// the process; it is only ever used through an attached JNI env.
unsafe impl Send for AndroidPlatform {}

impl AndroidPlatform {
    /// # Safety
    /// `vm` and `activity` must be the pointers from `AndroidApp::vm_as_ptr` / `activity_as_ptr`.
    pub unsafe fn new(root: PathBuf, vm: *mut std::ffi::c_void, activity: *mut std::ffi::c_void) -> Option<Self> {
        Some(AndroidPlatform {
            root,
            vm: JavaVM::from_raw(vm.cast()).ok()?,
            activity: activity as jni::sys::jobject,
            cache: Mutex::new(None),
            phones: Mutex::new(None),
            rumble: (0, None),
        })
    }

    fn query(&self) -> Option<Battery> {
        let mut env = self.vm.attach_current_thread().ok()?;
        let activity = unsafe { JObject::from_raw(self.activity) };
        let action = env.new_string("android.intent.action.BATTERY_CHANGED").ok()?;
        let filter = env
            .new_object(
                "android/content/IntentFilter",
                "(Ljava/lang/String;)V",
                &[JValue::Object(&action)],
            )
            .ok()?;
        let null = JObject::null();
        let intent = env
            .call_method(
                &activity,
                "registerReceiver",
                "(Landroid/content/BroadcastReceiver;Landroid/content/IntentFilter;)Landroid/content/Intent;",
                &[JValue::Object(&null), JValue::Object(&filter)],
            )
            .ok()?
            .l()
            .ok()?;
        if intent.is_null() {
            return None;
        }
        let mut int_extra = |name: &str| -> Option<i32> {
            let key = env.new_string(name).ok()?;
            env.call_method(
                &intent,
                "getIntExtra",
                "(Ljava/lang/String;I)I",
                &[JValue::Object(&key), JValue::Int(-1)],
            )
            .ok()?
            .i()
            .ok()
        };
        let level = int_extra("level")?;
        let scale = int_extra("scale")?;
        let status = int_extra("status")?;
        if level < 0 || scale <= 0 {
            return None;
        }
        // BatteryManager: 2 charging, 3 discharging, 4 not charging, 5 full.
        let charge = match status {
            2 => Charge::Charging,
            3 | 4 => Charge::Discharging,
            5 => Charge::Full,
            _ => Charge::Unknown,
        };
        Some(Battery {
            percent: (level * 100 / scale).clamp(0, 100) as u8,
            charge,
        })
    }

    /// Buzzes the phone for `ms` at `strength`/65535 (amplitude 1..=255), or stops it at 0.
    fn buzz(&self, strength: u16, ms: i64) {
        let Ok(mut env) = self.vm.attach_current_thread() else {
            return;
        };
        let activity = unsafe { JObject::from_raw(self.activity) };
        let run = (|| -> jni::errors::Result<()> {
            let name = env.new_string("vibrator")?;
            let vibrator = env
                .call_method(
                    &activity,
                    "getSystemService",
                    "(Ljava/lang/String;)Ljava/lang/Object;",
                    &[JValue::Object(&name)],
                )?
                .l()?;
            if vibrator.is_null() {
                return Ok(());
            }
            if strength == 0 {
                env.call_method(&vibrator, "cancel", "()V", &[])?;
                return Ok(());
            }
            let amplitude = ((u32::from(strength) * 255) / 65535).clamp(1, 255) as i32;
            let effect = env
                .call_static_method(
                    "android/os/VibrationEffect",
                    "createOneShot",
                    "(JI)Landroid/os/VibrationEffect;",
                    &[JValue::Long(ms), JValue::Int(amplitude)],
                )?
                .l()?;
            env.call_method(
                &vibrator,
                "vibrate",
                "(Landroid/os/VibrationEffect;)V",
                &[JValue::Object(&effect)],
            )?;
            Ok(())
        })();
        if run.is_err() {
            let _ = env.exception_clear();
        }
    }

    /// True while a wired or Bluetooth audio output is connected (`AudioManager.getDevices`).
    fn query_headphones(&self) -> bool {
        // AudioDeviceInfo types: wired headphones / headset, Bluetooth SCO / A2DP, USB headset,
        // BLE headset.
        const HEADPHONES: [i32; 6] = [3, 4, 7, 8, 22, 26];
        let Ok(mut env) = self.vm.attach_current_thread() else {
            return false;
        };
        let activity = unsafe { JObject::from_raw(self.activity) };
        let found = (|| -> jni::errors::Result<bool> {
            let name = env.new_string("audio")?;
            let audio = env
                .call_method(
                    &activity,
                    "getSystemService",
                    "(Ljava/lang/String;)Ljava/lang/Object;",
                    &[JValue::Object(&name)],
                )?
                .l()?;
            if audio.is_null() {
                return Ok(false);
            }
            // GET_DEVICES_OUTPUTS = 2
            let devices: JObjectArray = env
                .call_method(
                    &audio,
                    "getDevices",
                    "(I)[Landroid/media/AudioDeviceInfo;",
                    &[JValue::Int(2)],
                )?
                .l()?
                .into();
            let n = env.get_array_length(&devices)?;
            for i in 0..n {
                let device = env.get_object_array_element(&devices, i)?;
                let kind = env.call_method(&device, "getType", "()I", &[])?.i()?;
                env.delete_local_ref(device)?;
                if HEADPHONES.contains(&kind) {
                    return Ok(true);
                }
            }
            Ok(false)
        })();
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
        }
        found.unwrap_or(false)
    }

    fn cached(&self) -> Option<Battery> {
        let mut cache = self.cache.lock().ok()?;
        match *cache {
            Some((at, value)) if at.elapsed() < REFRESH => value,
            _ => {
                let value = self.query();
                *cache = Some((Instant::now(), value));
                value
            }
        }
    }
}

impl Platform for AndroidPlatform {
    fn set_backlight(&mut self, _step: u8) {}

    fn battery(&self) -> Option<Battery> {
        self.cached()
    }

    fn charge(&self) -> Charge {
        self.cached().map_or(Charge::Unknown, |b| b.charge)
    }

    fn set_led(&mut self, _state: LedState) {}

    fn headphones(&self) -> bool {
        const REFRESH: Duration = Duration::from_secs(2);
        let Ok(mut cache) = self.phones.lock() else {
            return false;
        };
        match *cache {
            Some((at, value)) if at.elapsed() < REFRESH => value,
            _ => {
                let value = self.query_headphones();
                *cache = Some((Instant::now(), value));
                value
            }
        }
    }

    fn poweroff(&mut self) -> ! {
        std::process::exit(0)
    }

    fn restart(&mut self) -> ! {
        std::process::exit(0)
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn now(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    /// Android owns the clock; slot's own clock screen is never shown (see `local_utc_offset_min`).
    fn set_clock(&mut self, _secs: i64) {}

    fn haptic(&mut self, percent: u8) {
        let strength = (0x7200u32 * u32::from(percent) / 100).min(0xffff) as u16;
        self.buzz(strength, 14);
    }

    fn set_rumble(&mut self, strength: u16) {
        const REFRESH: Duration = Duration::from_millis(80);
        let (last, at) = self.rumble;
        let due = at.is_none_or(|t| t.elapsed() >= REFRESH);
        match (strength, last) {
            (0, 0) => return,
            (0, _) => self.buzz(0, 0),
            (s, l) if s != l || due => self.buzz(s, 120),
            _ => return,
        }
        self.rumble = (strength, Some(Instant::now()));
    }
}

/// The device's current UTC offset in minutes, from bionic's tz database.
pub fn local_utc_offset_min() -> i16 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as libc::time_t)
        .unwrap_or(0);
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe { libc::localtime_r(&now, &mut tm) };
    (tm.tm_gmtoff / 60) as i16
}

/// `android.os.Build.MODEL` (e.g. "SM-F966B"), read once at boot for the About sticker.
///
/// # Safety
/// `vm` must be the pointer from `AndroidApp::vm_as_ptr`.
pub unsafe fn device_model(vm: *mut std::ffi::c_void) -> Option<String> {
    build_string(vm, "MODEL")
}

/// `android.os.Build.MANUFACTURER` (e.g. "samsung").
///
/// # Safety
/// As `device_model`.
pub unsafe fn device_maker(vm: *mut std::ffi::c_void) -> Option<String> {
    build_string(vm, "MANUFACTURER")
}

unsafe fn build_string(vm: *mut std::ffi::c_void, field: &str) -> Option<String> {
    let vm = JavaVM::from_raw(vm.cast()).ok()?;
    let mut env = vm.attach_current_thread().ok()?;
    let value = env
        .get_static_field("android/os/Build", field, "Ljava/lang/String;")
        .ok()?
        .l()
        .ok()?;
    let text: String = env.get_string((&value).into()).ok()?.into();
    Some(text)
}
