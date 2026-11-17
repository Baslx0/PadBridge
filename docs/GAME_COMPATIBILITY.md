# Game compatibility

PadBridge can create a virtual Xbox 360 controller from a selected Sony HID controller when the ViGEmBus driver is available. This path is optional and must be started from the Controllers page. PadBridge downloads a verified installer if the driver is missing; installation requires user action. Profile mapping currently affects the tester only; Xbox output uses raw Sony input. The USB behavior observed in GTA V through Epic/Rockstar comes from the game seeing the wired device.

## PadBridge Xbox output

1. Pair the PlayStation controller over Bluetooth. On PadBridge's Controllers page, select the Sony HID controller and click **Start Xbox output**. If ViGEmBus is missing, PadBridge automatically downloads the pinned 1.22.0 installer from the [original Nefarius release](https://github.com/nefarius/ViGEmBus/releases/tag/v1.22.0), checks its SHA-256, and shows **Open verified driver installer**. Complete the Windows installer, restart if asked, then click **Start Xbox output** again.
2. Check that Windows lists a virtual Xbox 360 controller in `joy.cpl`, then start GTA V and test it. Keep PadBridge open while playing.
3. Click **Stop Xbox output** when finished. Output also stops when the selected controller disconnects or stops reporting input.

This path has been built and unit tested, but has not been verified with the driver or GTA V on this PC. The physical controller may also be visible to games, so duplicate input is possible.

## Try GTA V from Epic/Rockstar over Bluetooth

1. Pair the PlayStation controller in Windows and check that PadBridge's tester sees its buttons.
2. Open Steam and confirm the controller appears under Steam > Settings > Controller. Enable PlayStation controller support if Steam offers that setting.
3. In Steam, use **Games > Add a Non-Steam Game to My Library**. Add the GTA V entry or its launcher, then launch it from Steam. In the shortcut's controller settings, enable Steam Input and choose a gamepad layout.
4. Test in game. If the game is started by Epic/Rockstar as a separate process and Steam's overlay does not attach, Steam Input may not reach it. Return to the existing USB method if that happens.

This is an optional external workaround, not a PadBridge feature. Steam's own documentation describes non-Steam game setup and warns that separate launchers and overlays can limit it. Steam Input can translate supported PlayStation controller input to XInput when its gamepad emulation is active.

Sources: https://help.steampowered.com/en/faqs/view/33E8-5EDF-24E6-4CFB ; https://partner.steamgames.com/doc/features/steam_controller/legacy_mode ; https://partner.steamgames.com/doc/features/steam_controller/steam_input_gamepad_emulation_bestpractices

## Driver status

ViGEmBus is archived. Microsoft VHF requires a kernel-mode HID source driver. A newer fork, LuminalVGBus, explicitly says its driver is unsigned and has never been loaded or runtime-tested. PadBridge downloads the official installer only after an Xbox output attempt finds the driver missing. It does not install the driver automatically. Virtual output requires the user to complete installation.

Sources: https://github.com/nefarius/ViGEmBus ; https://learn.microsoft.com/en-us/windows-hardware/drivers/hid/virtual-hid-framework--vhf- ; https://github.com/NortheBridge/LuminalVGBus
