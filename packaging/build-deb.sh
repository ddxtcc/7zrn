#!/bin/sh
set -eu

: "${root:?}" "${version:?}" "${bin:?}" "${out:?}" "${stage_root:?}"

command -v dpkg-deb >/dev/null 2>&1 || {
    echo "dpkg-deb is required" >&2
    exit 1
}

pkg=$stage_root/deb
rm -rf "$pkg"
mkdir -p "$pkg/DEBIAN" "$pkg/usr/bin" "$pkg/usr/share/applications" "$pkg/usr/share/icons"

install -Dm755 "$bin" "$pkg/usr/bin/sevenzrn"
install -Dm644 "$root/packaging/7zrn.desktop" "$pkg/usr/share/applications/7zrn.desktop"
cp -a "$root/assets/icons/hicolor" "$pkg/usr/share/icons/"
install -Dm644 "$root/LICENSE" "$pkg/usr/share/doc/sevenzrn/copyright"

installed_size=$(du -sk "$pkg/usr" | cut -f1)

cat > "$pkg/DEBIAN/control" <<EOF
Package: sevenzrn
Version: $version
Architecture: amd64
Maintainer: ddxtcc <ddxtcc@users.noreply.github.com>
Installed-Size: $installed_size
Depends: 7zip | p7zip-full, tar, libxkbcommon0, libxkbcommon-x11-0, libfreetype6, libxcb1, libxcb-randr0, libxcb-shm0, libxcb-xfixes0, libxcb-xkb1, libx11-6, libwayland-client0, libvulkan1, libfontconfig1, libegl1, libgl1, libpng16-16, zlib1g, libbz2-1.0, libbrotli1
Section: utils
Priority: optional
Homepage: https://github.com/ddxtcc/7zrn
Description: Native GPUI archive manager powered by 7-Zip and tar
 7zrn is a dual-pane archive manager for Linux with archive creation,
 extraction, encryption, previews, and desktop integration.
EOF

cat > "$pkg/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q /usr/share/icons/hicolor || true
fi
exit 0
EOF
chmod 755 "$pkg/DEBIAN/postinst"

dpkg-deb --build --root-owner-group "$pkg" "$out/sevenzrn_${version}_amd64.deb"
