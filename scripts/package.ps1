param([switch]$SkipBuild)

$ErrorActionPreference = 'Stop'
$repo = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Push-Location $repo
try {
    if (-not $SkipBuild) {
        cargo build --release --locked
        if ($LASTEXITCODE -ne 0) { throw 'Release build failed.' }
    }

    $metadata = cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Could not read Cargo package metadata.' }
    $package = $metadata.packages | Where-Object { $_.name -eq 'padbridge' } | Select-Object -First 1
    if (-not $package) { throw 'PadBridge package was not found.' }

    $exe = Join-Path $repo 'target\release\padbridge.exe'
    if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) { throw "Missing release executable: $exe" }

    $dist = Join-Path $repo 'dist'
    $name = "PadBridge-v$($package.version)-windows-x64"
    $stage = Join-Path $dist $name
    $distFull = [System.IO.Path]::GetFullPath($dist).TrimEnd('\')
    $stageFull = [System.IO.Path]::GetFullPath($stage)
    if (-not $stageFull.StartsWith($distFull + '\', [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Unsafe staging path: $stageFull"
    }
    New-Item -ItemType Directory -Force -Path $dist | Out-Null
    if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $stage | Out-Null

    Copy-Item -LiteralPath $exe -Destination (Join-Path $stage 'PadBridge.exe')
    Copy-Item -LiteralPath (Join-Path $repo 'README.md') -Destination $stage
    Copy-Item -LiteralPath (Join-Path $repo 'docs\GETTING_STARTED.md') -Destination $stage
    Copy-Item -LiteralPath (Join-Path $repo 'docs\GAME_COMPATIBILITY.md') -Destination $stage
    $viiper = Join-Path $stage 'third_party\VIIPER'
    $usbip = Join-Path $stage 'third_party\usbip-win2'
    New-Item -ItemType Directory -Force -Path $viiper, $usbip | Out-Null
    foreach ($file in @('LICENSE.txt', 'README.md', 'source-v0.9.0.zip')) {
        Copy-Item -LiteralPath (Join-Path $repo "third_party\VIIPER\$file") -Destination $viiper
    }
    foreach ($file in @('LICENSE.txt', 'README.md')) {
        Copy-Item -LiteralPath (Join-Path $repo "third_party\usbip-win2\$file") -Destination $usbip
    }

    $zip = Join-Path $dist "$name.zip"
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip -Force
    $hash = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
    $checksum = Join-Path $dist "$name.zip.sha256"
    Set-Content -LiteralPath $checksum -Value "$hash  $name.zip" -Encoding ascii
    Write-Output "Created $zip"
    Write-Output "SHA-256 $hash"
}
finally {
    Pop-Location
}
