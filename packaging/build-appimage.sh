#!/bin/sh
set -eu

: "${root:?}" "${version:?}" "${bin:?}" "${out:?}" "${stage_root:?}"

command -v curl >/dev/null 2>&1 || {
    echo "curl is required" >&2
    exit 1
}

work=$stage_root/appimage
tools=$work/tools
appdir=$work/AppDir

mkdir -p "$tools" "$out"

if [ ! -x "$tools/linuxdeploy-x86_64.AppImage" ]; then
    curl -fL --retry 3 -o "$tools/linuxdeploy-x86_64.AppImage" \
        https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
    chmod +x "$tools/linuxdeploy-x86_64.AppImage"
fi

if [ ! -x "$tools/appimagetool-x86_64.AppImage" ]; then
    curl -fL --retry 3 -o "$tools/appimagetool-x86_64.AppImage" \
        https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
    chmod +x "$tools/appimagetool-x86_64.AppImage"
fi

rm -rf "$appdir"
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/applications" "$appdir/usr/share/icons"

install -Dm755 "$bin" "$appdir/usr/bin/sevenzrn"
install -Dm644 "$root/packaging/7zrn.desktop" "$appdir/usr/share/applications/7zrn.desktop"
cp -a "$root/assets/icons/hicolor" "$appdir/usr/share/icons/"

APPIMAGE_EXTRACT_AND_RUN=1 "$tools/linuxdeploy-x86_64.AppImage" \
    --appdir "$appdir" \
    --executable "$appdir/usr/bin/sevenzrn" \
    --desktop-file "$appdir/usr/share/applications/7zrn.desktop" \
    --icon-file "$root/assets/icons/hicolor/512x512/apps/7zrn.png"

if [ ! -x "$appdir/AppRun" ]; then
    cat > "$appdir/AppRun" <<'EOF'
#!/bin/sh
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec "$here/usr/bin/sevenzrn" "$@"
EOF
    chmod +x "$appdir/AppRun"
fi

APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 "$tools/appimagetool-x86_64.AppImage" \
    "$appdir" "$out/sevenzrn-${version}-x86_64.AppImage"
