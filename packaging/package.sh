#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
version=${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -n 1)}
arch=${ARCH:-x86_64}
bin=${BIN:-$root/target/release/sevenzrn}
out=${OUT:-$root/dist}
stage_root=${STAGE_ROOT:-$root/.package}
stage=$stage_root/sevenzrn-linux-x86_64

if [ ! -x "$bin" ]; then
    echo "release binary not found: $bin" >&2
    exit 1
fi

rm -rf "$stage_root"
mkdir -p "$stage/icons" "$out"

install -Dm755 "$bin" "$stage/sevenzrn"
strip "$stage/sevenzrn"
bin=$stage/sevenzrn
install -Dm644 "$root/packaging/7zrn.desktop" "$stage/7zrn.desktop"
install -Dm755 "$root/packaging/install.sh" "$stage/install.sh"
cp -a "$root/assets/icons/hicolor" "$stage/icons/"
install -Dm644 "$root/README.md" "$stage/README.md"
install -Dm644 "$root/CHANGELOG.md" "$stage/CHANGELOG.md"
install -Dm644 "$root/LICENSE" "$stage/LICENSE"
tar -C "$stage_root" -czf "$out/sevenzrn-linux-x86_64.tar.gz" sevenzrn-linux-x86_64

export root version arch bin out stage_root
sh "$root/packaging/build-deb.sh"
sh "$root/packaging/build-rpm.sh"
sh "$root/packaging/build-appimage.sh"

(
    cd "$out"
    sha256sum ./*.deb ./*.rpm ./*.AppImage ./*.tar.gz > SHA256SUMS
)
