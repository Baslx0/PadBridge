# Game compatibility

PadBridge currently **monitors** controller input. It does not translate a Bluetooth DualSense/DualShock into an Xbox controller or send remapped input to games. A game that accepts the physical controller directly may work; a game that needs XInput may not. The USB behavior observed in GTA V through Epic/Rockstar comes from the game seeing the wired device, not from PadBridge.

## Try GTA V from Epic/Rockstar over Bluetooth

1. Pair the PlayStation controller in Windows and check that PadBridge's tester sees its buttons.
2. Open Steam and confirm the controller appears under Steam > Settings > Controller. Enable PlayStation controller support if Steam offers that setting.
3. In Steam, use **Games > Add a Non-Steam Game to My Library**. Add the GTA V entry or its launcher, then launch it from Steam. In the shortcut's controller settings, enable Steam Input and choose a gamepad layout.
4. Test in game. If the game is started by Epic/Rockstar as a separate process and Steam's overlay does not attach, Steam Input may not reach it. Return to the existing USB method if that happens.

This is an optional external workaround, not a PadBridge feature. Steam's own documentation describes non-Steam game setup and warns that separate launchers and overlays can limit it. Steam Input can translate supported PlayStation controller input to XInput when its gamepad emulation is active.

Sources: https://help.steampowered.com/en/faqs/view/33E8-5EDF-24E6-4CFB ; https://partner.steamgames.com/doc/features/steam_controller/legacy_mode ; https://partner.steamgames.com/doc/features/steam_controller/steam_input_gamepad_emulation_bestpractices

## Native PadBridge output

A trustworthy built-in virtual Xbox backend requires a verified, signed driver and actual game testing. ViGEmBus is archived. Microsoft VHF requires a kernel-mode HID source driver. A newer fork, LuminalVGBus, explicitly says its driver is unsigned and has never been loaded or runtime-tested. PadBridge does not download, install, or depend on any of these drivers. Until a suitable backend is verified, game output stays unavailable.

Sources: https://github.com/nefarius/ViGEmBus ; https://learn.microsoft.com/en-us/windows-hardware/drivers/hid/virtual-hid-framework--vhf- ; https://github.com/NortheBridge/LuminalVGBus
