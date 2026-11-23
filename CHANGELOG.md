# Changelog

## Unreleased

- Old-school NFO viewer with CP437 decoding, ANSI colors, no-wrap 80-column layout, and automatic full-screen root-NFO preview
- RAR and RAR5 detection and extraction through 7z
- Hex preview for binary files with offsets and ASCII rendering
- SHA-256, SHA-1, MD5, and CRC32 checksums for files and archive entries
- Copy checksums as SHA256SUMS, save a SHA256SUMS file, and verify checksum manifests

## 0.2.0 - 2026-11-22

- Rendered Markdown previews with headings, lists, tables, quotes, links, and fenced code
- Code previews with line numbers, language badges, tab expansion, and expanded syntax support
- Shebang and filename detection for extensionless scripts and common build files
- Archive rename, move, copy, and folder creation
- Tabs, session restore, and nested archive opening
- Recursive name and content search, plus a command palette
- Cancelable jobs with an operation log
- Archive creation compression, password, split, and exclude options
- Archive properties with unit-aware sizes, compression bars, file-type distribution, and largest-file graphs
- Archive test reports with formatted sizes and a details dashboard
- PDF previews, image zoom, clickable Markdown links, local images, code wrapping, and copy
- Resizable and persisted preview pane
- AUR VCS package and Flatpak packaging

## 0.1.0 - 2026-11-22

Initial release.

- Dual-pane GPUI archive manager for 7z, zip, tar, and compressed tar archives
- Encrypted archive support with password prompts and retry
- Add, delete, extract, test, and create archives
- Inline preview pane for text, images, and syntax-highlighted JSON and code
- Automatic click preview up to 4 MiB, with a Settings toggle
- Open files directly, extracting only the selected archive entry
- Context menus for both panes
- Drag and drop between panes and external file drops
- Resizable panes and columns, persisted across restarts
- Six color themes
- Desktop installer with archive MIME defaults
- Debian, RPM, AppImage, and tarball release packages
