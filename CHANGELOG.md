# Changelog

## 0.8.0 - 2026-11-23

- Keep automatic NFO previews in the bottom pane instead of opening a modal
- Move file previews and filesystem search off the UI thread
- Pass archive passwords through stdin instead of process arguments
- Preserve AES-256 ZIP encryption during rebuilds
- Prevent ZIP edit downgrades, split-volume edits, and unsafe extracted-file following
- Honor tar backups and the configured edit compression when adding files
- Fix conversion output validation and split-volume output handling
- Require contiguous numbered volumes before merging
- Use private runtime cache directories for extracted previews and temporary work
- Fix modal field switching, password masking, tab focus, cursor repaint, and modal scrolling
- Reap external open processes and keep tab audio state in sync
- Clear stale previews and edits when opening archives or showing job result modals
- Apply configured new-archive defaults and hidden-file state across tabs

## 0.7.1 - 2026-11-23

- Load automatic NFO previews in the bottom pane instead of opening a modal

## 0.7.0 - 2026-11-23

- Configurable general and audio preview size limits
- Configurable text and hex preview sizes
- Configurable PDF render DPI
- Choose ffplay or mpv for audio playback
- Toggle embedded audio cover art

## 0.6.1 - 2026-11-23

- Raise general preview limit to 64 MiB and audio previews to 1 GiB

## 0.6.0 - 2026-11-23

- Audio previews with tags, embedded cover art, technical metadata, and play/stop controls
- Audio autoplay setting

## 0.5.0 - 2026-11-23

- Expanded settings for preview, editing, moves, defaults, backups, and session restore
- Preserve configured compression and optional .bak backups during archive edits
- Convert and recompress archives with format, compression, password, and split options
- Merge numbered archive volumes
- Archive checksum manifest verification and adding SHA256SUMS to archives
- Cross-pane move with source deletion confirmation

## 0.4.0 - 2026-11-23

- Verify checksum manifests inside archives without extracting the whole archive
- Add SHA256SUMS files to the open archive from the checksum modal
- Move Files selections into the archive with source deletion confirmation
- Move archive entries to Files, extracting and removing the archived entries

## 0.3.0 - 2026-11-23

- Old-school NFO viewer with CP437 decoding, ANSI colors, no-wrap 80-column layout, and automatic full-screen root-NFO preview
- RAR and RAR5 detection and extraction through 7z
- Hex preview for binary files with offsets, ASCII rendering, and byte-class colors
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
