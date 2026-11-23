# Arch packaging

`PKGBUILD` builds `sevenzrn-git`, a VCS package that tracks
`https://github.com/ddxtcc/7zrn.git` on the main branch. The v0.8.2 tag can be
used as the source for a stable PKGBUILD.

Runtime dependencies include `7zip`, `tar`, and the GPUI graphics and input
libraries (`freetype2`, `libxcb`, `libxkbcommon`, `libxkbcommon-x11`, `libx11`,
`wayland`, and `vulkan-icd-loader`). The optional compressors `gzip`, `bzip2`,
`xz`, and `zstd` are used for compressed tar and single stream archives.

## Build and install

Run the build from this directory:

```sh
cd packaging/aur
makepkg -si
```

To build the package without installing it:

```sh
makepkg -f
sudo pacman -U sevenzrn-git-*.pkg.tar.zst
```

`pkgver()` derives the package version from `git describe`, so the version is
updated each time the source is refreshed.

## Publish to the AUR

Create an AUR repository named `sevenzrn-git`, copy `PKGBUILD` and this README,
regenerate `.SRCINFO`, and push:

```sh
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD README.md .SRCINFO
git commit -m "Update sevenzrn-git"
git push
```

Run `namcap PKGBUILD` and `namcap sevenzrn-git-*.pkg.tar.zst` before uploading.
