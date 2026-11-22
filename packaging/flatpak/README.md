# Flatpak packaging

The application ID is `io.github.ddxtcc.SevenZrn`. The manifest builds the
Rust application with `cargo build --release --locked` from the repository and
bundles a 7-Zip console binary built from the official 7-Zip source archive.
The bundled binary is installed as `/app/bin/7zz` with a `/app/bin/7z` symlink,
so the application finds `7z` inside the sandbox.

`cargo-sources.json` was generated from `Cargo.lock` with
`flatpak-cargo-generator.py`. Regenerate it whenever `Cargo.lock` changes:

```sh
flatpak-cargo-generator.py Cargo.lock -o packaging/flatpak/cargo-sources.json
```

## Prerequisites

Install Flatpak, flatpak-builder, the GNOME runtime and SDK, and the Rust SDK
extension:

```sh
flatpak install --user flathub org.gnome.Platform//48 org.gnome.Sdk//48
flatpak install --user flathub org.freedesktop.Sdk.Extension.rust-stable//24.08
```

The Rust extension branch must match the freedesktop SDK branch used by the
GNOME runtime. GNOME 48 is based on freedesktop SDK 24.08.

## Build and install

From the repository root:

```sh
flatpak-builder --user --install --force-clean build-dir packaging/flatpak/io.github.ddxtcc.SevenZrn.yml
flatpak run io.github.ddxtcc.SevenZrn
```

Build a bundle without installing:

```sh
flatpak-builder --force-clean --repo=repo build-dir packaging/flatpak/io.github.ddxtcc.SevenZrn.yml
flatpak build-bundle repo sevenzrn.flatpak io.github.ddxtcc.SevenZrn
```

The manifest uses a `dir` source for local builds. For a Flathub submission,
replace it with a pinned `git` source at the release commit and keep the same
build commands.

## Verification

The manifest and metadata were syntax checked with PyYAML,
`desktop-file-validate`, and `appstreamcli validate`. The bundled 7-Zip build
command was run against the 7-Zip 26.06 source archive and produced a working
`7zz` binary. A full `flatpak-builder` build was not run here because
flatpak-builder and the Flatpak runtimes are not installed in this environment.

## Runtime limitations

- 7-Zip is bundled from source, so the sandbox does not need a host `7z`.
- `tar`, `gzip`, `bzip2`, `xz`, and `zstd` are not bundled by this manifest.
  Tar and compressed tar support depends on those executables being present in
  the Flatpak runtime. Their presence in `org.gnome.Platform` was not verified
  in this environment.
- The sandbox has read and write access to the home directory and to
  `/run/media`, `/media`, and `/mnt` for removable and mounted volumes.
  Other host paths are not visible.
- The application uses `xdg-open` or `gio open` for revealing extracted files.
  This depends on the runtime and the desktop portal.
