# Getting started on Windows

## Download and run

Download `PadBridge-v*-windows-x64.zip` from [PadBridge Releases](https://github.com/Baslx0/PadBridge/releases). Extract the full ZIP to a folder and double-click `PadBridge.exe`. Do not run the executable from inside the ZIP. Windows 10/11 x64 is the intended platform.

If there is no release ZIP yet, use the build steps below. Development builds from the **Actions** page are also available as artifacts after a successful Windows build, though GitHub may require you to sign in to download them.

## Connect a controller and play

1. Pair a supported PlayStation controller in Windows Bluetooth settings, or connect it by USB.
2. On **Controllers**, select the Sony controller. If it is absent, reconnect it and check whether Windows sees it.
3. On **Input tester**, press each control and watch the controller drawing. This tests the physical input before starting a game.
4. Back on **Controllers**, click **Start Xbox output**. If prompted, open the bundled USB/IP installer and approve the Windows driver installation. It can briefly restart USB hubs. Restart Windows if the installer asks, then start output again.
5. Wait until PadBridge says **Active** and shows an XInput slot. Start the game after this point. Leave PadBridge running while you play.

If a game does not respond, check that output is **Active**, close and reopen the game, and avoid connecting the same controller by both USB and Bluetooth. See [Game compatibility](GAME_COMPATIBILITY.md) for GTA V notes. The physical controller may also appear in a game and cause duplicate input.

## Build from source

Install [Rust for Windows](https://rust-lang.org/tools/install/) with the `x86_64-pc-windows-msvc` toolchain, plus the Visual Studio C++ build tools and Windows SDK required by that toolchain. Clone this repository using its **Code** button, open PowerShell in the clone, and run:

```powershell
cargo build --release --locked
```

Run `target\release\padbridge.exe`. To create a distributable ZIP containing the required third party notices and source materials, run:

```powershell
powershell -NoProfile -File .\scripts\package.ps1 -SkipBuild
```

The result is under `dist\PadBridge-v*-windows-x64.zip`, with a `.sha256` checksum alongside it. If PowerShell blocks scripts under your policy, run `cargo build --release --locked` and launch the executable directly; packaging is optional for local use.
