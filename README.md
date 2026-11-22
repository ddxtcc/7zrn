# 7zrn

A native GPUI archive manager for Linux, powered by the 7-Zip and tar command line tools.

## Features

- Dual-pane Files and Archive browsing
- 7z, zip, tar, compressed tar, gzip, bzip2, xz, zstd, rar, iso, and deb support through 7z and tar
- Encrypted archive passwords, with retry on wrong passwords
- Add, delete, extract, test, and create archives
- Extract to a chosen folder, with optional automatic reveal in Files
- Inline text and image previews, with JSON and code syntax highlighting
- Automatic preview of clicked files up to 4 MiB, configurable in Settings
- Open files directly, extracting only the selected archive entry when needed
- Right-click context menus in both panes
- Drag files between panes and accept external file drops
- Resizable panes and columns, with persisted sizes
- Six color themes: Dark, Light, Nord, Gruvbox, Dracula, and Solarized
- Desktop integration and archive MIME defaults through the local installer

## Requirements

- Rust stable
- `7z` (7zip on Ubuntu 24.04) and `tar`
- `gzip`, `bzip2`, `xz`, and `zstd` for compressed tar and single stream archives

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
