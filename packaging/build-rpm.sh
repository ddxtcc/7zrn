#!/bin/sh
set -eu

: "${root:?}" "${version:?}" "${bin:?}" "${out:?}" "${stage_root:?}"

command -v rpmbuild >/dev/null 2>&1 || {
    echo "rpmbuild is required" >&2
    exit 1
}

work=$stage_root/rpmbuild
rm -rf "$work"
mkdir -p "$work"

rpmbuild -bb \
    --define "_topdir $work" \
    --define "version $version" \
    --define "repo $root" \
    --define "bin $bin" \
    "$root/packaging/sevenzrn.spec"

cp "$work/RPMS/x86_64/sevenzrn-${version}-1.x86_64.rpm" "$out/"
