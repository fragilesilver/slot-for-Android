#!/bin/sh
# Compile the Java helper and put classes.dex into an already-built APK, then align and sign it
# again.   android/build-dex.sh <apk> <keystore> <password-file|-> [alias]
# Needs a JDK (javac) and the Android build-tools; ~/.local/jdk is used when javac is not on PATH.
set -e
APK="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")" KS="$2" PWF="$3"
HERE="$(cd "$(dirname "$0")" && pwd)"
SDK="${ANDROID_HOME:-$HOME/Android/Sdk}"
BT="$(ls -d "$SDK"/build-tools/* | sort -V | tail -1)"
PLATFORM="$(ls -d "$SDK"/platforms/android-* | sort -V | tail -1)"
JAVAC="$(command -v javac || echo "$HOME/.local/jdk/bin/javac")"
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT
"$JAVAC" -source 8 -target 8 -Xlint:-options -cp "$PLATFORM/android.jar" -d "$OUT/classes" \
    $(find "$HERE/java" -name '*.java')
"$BT/d8" --release --min-api 26 --lib "$PLATFORM/android.jar" --output "$OUT" \
    $(find "$OUT/classes" -name '*.class')
( cd "$OUT" && "$BT/aapt" add "$APK" classes.dex >/dev/null )
"$BT/zipalign" -f -p 4 "$APK" "$OUT/aligned.apk"
mv "$OUT/aligned.apk" "$APK"
if [ "$PWF" = "-" ]; then PASS="pass:android"; else PASS="file:$PWF"; fi
"$BT/apksigner" sign --ks "$KS" --ks-pass "$PASS" "$APK"
