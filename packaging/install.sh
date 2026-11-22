#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
prefix=${PREFIX:-$HOME/.local}
data=${XDG_DATA_HOME:-$prefix/share}
repo=$(CDPATH= cd -- "$root/.." && pwd)

if [ "$#" -ge 1 ]; then
    binary=$1
elif [ -x "$root/sevenzrn" ]; then
    binary=$root/sevenzrn
elif [ -x "$root/../target/release/sevenzrn" ]; then
    binary=$root/../target/release/sevenzrn
else
    binary=
fi

if [ "$#" -lt 1 ] && [ "$root" = "$repo/packaging" ] && [ -f "$repo/Cargo.toml" ] && command -v cargo >/dev/null 2>&1; then
    echo "Building sevenzrn..."
    (cd "$repo" && cargo build --release --locked)
    binary=$repo/target/release/sevenzrn
fi

if [ -z "$binary" ] || [ ! -x "$binary" ]; then
    echo "sevenzrn binary not found" >&2
    echo "Build it with: cargo build --release" >&2
    exit 1
fi

if [ -f "$root/7zrn.desktop" ]; then
    desktop=$root/7zrn.desktop
else
    desktop=$repo/packaging/7zrn.desktop
fi

if [ -d "$root/icons/hicolor" ]; then
    icons=$root/icons/hicolor
else
    icons=$repo/assets/icons/hicolor
fi

if [ ! -f "$desktop" ]; then
    echo "Desktop entry not found: $desktop" >&2
    exit 1
fi

install -Dm755 "$binary" "$prefix/bin/sevenzrn"
desktop_target="$data/applications/7zrn.desktop"
install -Dm644 "$desktop" "$desktop_target"
sed -i "s|^Exec=.*|Exec=$prefix/bin/sevenzrn %f|" "$desktop_target"
sed -i "s|^TryExec=.*|TryExec=$prefix/bin/sevenzrn|" "$desktop_target"

if [ -f "$icons/scalable/apps/7zrn.svg" ]; then
    install -Dm644 "$icons/scalable/apps/7zrn.svg" "$data/icons/hicolor/scalable/apps/7zrn.svg"
fi
for size in 16x16 24x24 32x32 48x48 64x64 128x128 256x256 512x512; do
    if [ -f "$icons/$size/apps/7zrn.png" ]; then
        install -Dm644 "$icons/$size/apps/7zrn.png" "$data/icons/hicolor/$size/apps/7zrn.png"
    fi
done

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$data/applications" >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q "$data/icons/hicolor" >/dev/null 2>&1 || true
fi

if command -v xdg-mime >/dev/null 2>&1; then
    mimes=$(sed -n 's|^MimeType=||p' "$desktop_target" | tr ';' '\n' | sed '/^$/d')
    count=0
    for mime in $mimes; do
        if XDG_DATA_HOME="$data" xdg-mime default 7zrn.desktop "$mime" >/dev/null 2>&1; then
            count=$((count + 1))
        fi
    done
    echo "Registered 7zrn for $count archive MIME type(s)."
fi

for tool in 7z tar; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "warning: $tool is required at runtime" >&2
    fi
done

echo "Installed 7zrn to $prefix/bin/sevenzrn"
echo "Desktop entry: $desktop_target"
