# Build the helper in release mode and put a zip plus SHA-256 checksums in dist\ (Windows).
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot
cargo build --release
$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.*)"' | Select-Object -First 1).Matches.Groups[1].Value
$name = "ec-extension-ofx-$version-windows-x86_64"
Remove-Item -Recurse -Force dist -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "dist\$name" | Out-Null
Copy-Item target\release\effectcraft-helper-ofx.exe, README.md, LICENSE-MIT, LICENSE-APACHE, ATTRIBUTION.md "dist\$name\"
Compress-Archive -Path "dist\$name" -DestinationPath "dist\$name.zip"
$hash = (Get-FileHash "dist\$name.zip" -Algorithm SHA256).Hash.ToLower()
"$hash  $name.zip" | Set-Content dist\SHA256SUMS.txt
Get-Content dist\SHA256SUMS.txt
