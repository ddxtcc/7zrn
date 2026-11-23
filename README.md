# 7zrn

A native GPUI archive manager for Linux, powered by the 7-Zip and tar command line tools.

## Features

- Dual-pane Files and Archive browsing
- 7z, zip, tar, compressed tar, gzip, bzip2, xz, zstd, rar, iso, and deb support through 7z and tar
- Encrypted archive passwords, with retry on wrong passwords
- Add, delete, extract, test, and create archives
- Rename, move, copy, and create folders inside archives
- Tabs with session restore and nested archive browsing
- Recursive name and content search, plus a command palette
- Cancelable long-running jobs with an operation log
- Extract to a chosen folder, with optional automatic reveal in Files
- Inline text, rendered Markdown, and image previews
- Syntax-highlighted code previews with line numbers, language badges, and shebang detection
- Markdown headings, lists, tables, quotes, clickable links, local images, and syntax-highlighted fenced code
- PDF and image previews with zoom, plus a resizable preview pane
- Old-school NFO viewer with CP437 and ANSI decoding, fixed 80-column layout, automatic root-NFO preview
- RAR and RAR5 listing and extraction through 7z (read-only)
- Hex preview for binary files with offsets, ASCII side channel, and byte-class colors
- Audio previews with tags, duration, bitrate, codec, sample rate, channels, cover art, and play/stop controls
- SHA-256, SHA-1, MD5, and CRC32 checksums with copy, SHA256SUMS save, and manifest verification
- Verify checksum manifests inside archives and add SHA256SUMS to archives
- Cross-pane move with confirmation for Files to archive and archive to Files
- Archive conversion and recompression with format, compression, password, and split options
- Merge numbered archive volumes
- Expanded settings for preview limits, text and hex sizes, PDF DPI, audio playback, editing, moves, defaults, backups, and session restore
- Automatic previews with configurable general, audio, text, and hex size limits
- Open files directly, extracting only the selected archive entry when needed
- Right-click context menus in both panes
- Drag files between panes and accept external file drops
- Archive creation options for compression, passwords, split volumes, and excludes
- Archive properties with unit-aware sizes, compression ratio, type distribution, and largest files
- Test reports include formatted sizes and a details dashboard
- Resizable panes, columns, and preview, with persisted sizes
- Six color themes: Dark, Light, Nord, Gruvbox, Dracula, and Solarized
- Desktop integration and archive MIME defaults through the local installer

## Requirements

- Rust stable
- `7z` (7zip on Ubuntu 24.04) and `tar`
- `gzip`, `bzip2`, `xz`, and `zstd` for compressed tar and single stream archives
- `pdftoppm` from poppler-utils for PDF previews
- `ffmpeg` and `ffplay` for audio metadata, cover art, and playback (or `mpv` for playback)

## Build and run

```sh
cargo run --release
```

## Install locally

```sh
sh packaging/install.sh
```

The installer builds the release binary from a source checkout, installs it to `$HOME/.local/bin`,
installs the desktop entry and icons, and registers `7zrn` as the default handler for
the archive MIME types listed in `packaging/7zrn.desktop`.

Override the install locations with `PREFIX` and `XDG_DATA_HOME`:

```sh
PREFIX="$HOME/.local" XDG_DATA_HOME="$HOME/.local/share" sh packaging/install.sh
```

A release tarball can be installed with the bundled script:

```sh
tar -xzf sevenzrn-linux-x86_64.tar.gz
cd sevenzrn-linux-x86_64
./install.sh
```

## Release packages

Tagged releases include:

- `sevenzrn_<version>_amd64.deb`
- `sevenzrn-<version>-1.x86_64.rpm`
- `sevenzrn-<version>-x86_64.AppImage`
- `sevenzrn-linux-x86_64.tar.gz`
- `SHA256SUMS`

The deb and RPM packages install the binary, desktop entry, and icons.
The AppImage and tarball use the host `7z` and `tar` tools.

## Arch and Flatpak packaging

Arch users can build the VCS package from `packaging/aur`:

```sh
cd packaging/aur
makepkg -si
```

The package is named `sevenzrn-git` and tracks the main branch. A stable
PKGBUILD can be based on the v0.7.1 tag. It depends on `7zip`, `tar`, and the
GPUI runtime libraries.

Flatpak packaging is in `packaging/flatpak`. See
[`packaging/flatpak/README.md`](packaging/flatpak/README.md) for prerequisites,
build commands, and runtime limitations. The Flatpak bundles 7-Zip from source
and uses the application ID `io.github.ddxtcc.SevenZrn`.
