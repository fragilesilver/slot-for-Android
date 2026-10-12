#!/bin/sh
# Build the patched libretro gpSP for Android arm64.   cores/gpsp/build-android.sh COMMIT WORKDIR OUTDIR
# Same source + patch + `platform=arm64` recipe as build.sh (dynarec, mmap'd JIT cache),
# cross-built with the NDK clang. Pass DYNAREC=0 to try the interpreter-only fallback.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
commit="$1" work="$2" out="$3"
ndk="${ANDROID_NDK_HOME:-$HOME/Android/Sdk/ndk/27.2.12479018}"
bin="$ndk/toolchains/llvm/prebuilt/linux-x86_64/bin"
src="$work/gpsp"
mkdir -p "$src" "$out"
[ -d "$src/.git" ] || git init -q "$src"
git -C "$src" cat-file -e "$commit^{commit}" 2>/dev/null ||
	git -C "$src" fetch -q --depth 1 https://github.com/libretro/gpsp "$commit"
git -C "$src" checkout -q --force --detach "$commit"
git -C "$src" clean -q -fdx
for p in "$here"/*.patch; do git -C "$src" apply "$p"; done
extra=""
[ "${DYNAREC:-1}" = 0 ] && extra="HAVE_DYNAREC=0"
make -C "$src" platform=arm64 $extra \
	CC="$bin/aarch64-linux-android26-clang" CXX="$bin/aarch64-linux-android26-clang++" \
	AR="$bin/llvm-ar" \
	LDFLAGS="-Wl,-z,max-page-size=16384" \
	GIT_VERSION="\"$(printf %s "$commit" | cut -c1-7)\"" \
	-j"$(nproc)"
cp "$src/gpsp_libretro.so" "$out/gpsp_libretro.so"
