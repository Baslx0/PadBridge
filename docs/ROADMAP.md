# Roadmap

Virtual Xbox 360 output through VIIPER and usbip-win2, live Sony HID input, local profiles, and a controller tester are implemented. GTA V through Epic/Rockstar has been confirmed on one user's setup.

Next areas of work:

1. Windows native device inventory and reliable per-unit identifiers; deduplicate API views.
2. Move gilrs polling and power queries to a worker with bounded UI updates.
3. Broader hardware testing with DualSense, DualShock 4, GameSir modes, XInput, and multiple identical controllers.
4. Add per-button remapping and connect profile settings to virtual output.
5. Keep the virtual output components and Windows build pipeline current as upstream projects change.
