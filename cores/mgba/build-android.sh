#!/bin/sh
# Build the patched libretro mGBA for Android arm64.   cores/mgba/build-android.sh COMMIT WORKDIR OUTDIR
# Same source + patches as build.sh, cross-built with the NDK CMake toolchain.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
commit="$1" work="$2" out="$3"
ndk="${ANDROID_NDK_HOME:-$HOME/Android/Sdk/ndk/27.2.12479018}"
src="$work/mgba" obj="$work/build-android"
mkdir -p "$src" "$out"
[ -d "$src/.git" ] || git init -q "$src"
git -C "$src" cat-file -e "$commit^{commit}" 2>/dev/null ||
	git -C "$src" fetch -q --depth 1 https://github.com/libretro/mgba "$commit"
git -C "$src" checkout -q --force --detach "$commit"
git -C "$src" clean -q -fdx
for p in "$here"/*.patch; do git -C "$src" apply "$p"; done
cmake -S "$src" -B "$obj" -DCMAKE_TOOLCHAIN_FILE="$ndk/build/cmake/android.toolchain.cmake" \
	-DANDROID_ABI=arm64-v8a -DANDROID_PLATFORM=android-26 -DCMAKE_BUILD_TYPE=Release \
	-DLIBMGBA_ONLY=ON -DBUILD_LIBRETRO=ON \
	-DCMAKE_SHARED_LINKER_FLAGS="-Wl,-z,max-page-size=16384"
cmake --build "$obj" --target mgba_libretro --parallel "$(nproc)"
cp "$obj/mgba_libretro.so" "$out/libmgba_libretro.so"
