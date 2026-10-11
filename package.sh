#!/usr/bin/env sh
# Build the helper in release mode and put a zip plus SHA-256 checksums in dist/.
set -eu
cd "$(dirname "$0")"
cargo build --release
case "$(uname -s)" in
  Darwin) os=macos ;;
  Linux) os=linux ;;
  *) os=windows ;;
esac
exe=effectcraft-helper-ofx
[ "$os" = windows ] && exe=effectcraft-helper-ofx.exe
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
name="ec-extension-ofx-$version-$os-$(uname -m)"
rm -rf dist && mkdir -p "dist/$name"
cp "target/release/$exe" README.md LICENSE-MIT LICENSE-APACHE ATTRIBUTION.md "dist/$name/"
(cd dist && if command -v zip >/dev/null; then zip -qr "$name.zip" "$name"; else tar czf "$name.tar.gz" "$name"; fi)
(cd dist && (sha256sum ./*.zip ./*.tar.gz 2>/dev/null || shasum -a 256 ./*.zip ./*.tar.gz 2>/dev/null) > SHA256SUMS.txt) || true
cat dist/SHA256SUMS.txt
