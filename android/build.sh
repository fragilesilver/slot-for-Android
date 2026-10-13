#!/bin/sh
# Build (and optionally install + launch) the debug APK.   android/build.sh [run]
# RUSTFLAGS is set here because cargo-apk ignores .cargo/config.toml rustflags; the
# 16 KB max-page-size is required by Android 15+/16 devices (else a compat dialog shows).
set -e
cd "$(dirname "$0")/../crates/slot"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}" ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/27.2.12479018}"
export RUSTFLAGS="-C link-arg=-Wl,-z,max-page-size=16384 ${RUSTFLAGS:-}"
# Android's installer unpacks only lib*.so from an APK, so the cores must carry that prefix or
# they are never extracted (the app then falls back to its test pattern).
for c in mgba gpsp; do
    d=android-libs/arm64-v8a
    if [ -f "$d/${c}_libretro.so" ] && [ ! -f "$d/lib${c}_libretro.so" ]; then
        mv "$d/${c}_libretro.so" "$d/lib${c}_libretro.so"
    fi
done
[ -f android-libs/arm64-v8a/libmgba_libretro.so ] || echo "note: no core in crates/slot/android-libs (run cores/mgba/build-android.sh)" >&2
PROFILE_FLAG=""
if [ "${SLOT_RELEASE:-}" = 1 ]; then
    PROFILE_FLAG="--release"
    # A release is signed with the keystore in ~/.android (never in the repo). Without it the
    # build falls back to the debug key from Cargo.toml, which is fine for testing only.
    KS="${SLOT_KEYSTORE:-$HOME/.android/slot-release.jks}"
    PWF="${SLOT_KEYSTORE_PASSWORD_FILE:-$HOME/.android/slot-release.password}"
    if [ -f "$KS" ] && [ -f "$PWF" ]; then
        export CARGO_APK_RELEASE_KEYSTORE="$KS"
        export CARGO_APK_RELEASE_KEYSTORE_PASSWORD="$(cat "$PWF")"
        echo "signing with $KS" >&2
    else
        echo "warning: no release keystore at $KS, signing with the debug key" >&2
    fi
fi
cargo apk build $PROFILE_FLAG --lib --no-default-features --features android
# The Java helper (gallery picker) goes in as classes.dex; the APK is aligned and signed again
# with the same key.
if [ "${SLOT_RELEASE:-}" = 1 ] && [ -n "${CARGO_APK_RELEASE_KEYSTORE:-}" ]; then
    DEX_APK=../../target/release/apk/slot-android.apk
    DEX_KS="$CARGO_APK_RELEASE_KEYSTORE" DEX_PW="$PWF"
elif [ "${SLOT_RELEASE:-}" = 1 ]; then
    DEX_APK=../../target/release/apk/slot-android.apk
    DEX_KS="$HOME/.android/debug.keystore" DEX_PW=-
else
    DEX_APK=../../target/debug/apk/slot-android.apk
    DEX_KS="$HOME/.android/debug.keystore" DEX_PW=-
fi
../../android/build-dex.sh "$DEX_APK" "$DEX_KS" "$DEX_PW"
if [ "${1:-}" = run ]; then
    ADB="$ANDROID_HOME/platform-tools/adb"
    APK=../../target/debug/apk/slot-android.apk
    [ "${SLOT_RELEASE:-}" = 1 ] && APK=../../target/release/apk/slot-android.apk
    "$ADB" install -r "$APK"
    "$ADB" shell am force-stop com.fragilesilver.slot
    "$ADB" shell am start -n com.fragilesilver.slot/com.fragilesilver.slot.SlotActivity
fi
