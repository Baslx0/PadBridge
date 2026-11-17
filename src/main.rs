#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Color32, RichText};
use gilrs::{Axis, Button, Gilrs, PowerInfo};
use std::time::Duration;

fn battery_label(power: PowerInfo) -> String {
    match power {
        PowerInfo::Unknown => "Unavailable".into(),
        PowerInfo::Wired => "Wired (battery not reported)".into(),
        PowerInfo::Discharging(percent) => format!("{percent}% - discharging"),
        PowerInfo::Charging(percent) => format!("{percent}% - charging"),
        PowerInfo::Charged => "Fully charged".into(),
    }
}

struct PadBridge {
    gilrs: Option<Gilrs>,
    error: Option<String>,
    deadzone: f32,
}

impl Default for PadBridge {
    fn default() -> Self {
        match Gilrs::new() {
            Ok(gilrs) => Self { gilrs: Some(gilrs), error: None, deadzone: 0.10 },
            Err(err) => Self { gilrs: None, error: Some(err.to_string()), deadzone: 0.10 },
        }
    }
}

impl eframe::App for PadBridge {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(50));

        if let Some(gilrs) = self.gilrs.as_mut() {
            while gilrs.next_event().is_some() {}
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(RichText::new("PadBridge").size(30.0).strong());
            ui.label("Local controller monitor  •  Preview v0.1");
            ui.add_space(12.0);
            ui.label("USB / Bluetooth devices exposed as gamepads by Windows");
            ui.label("No cloud, background service, admin privileges or driver installation.");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Display deadzone:");
                ui.add(egui::Slider::new(&mut self.deadzone, 0.0..=0.35).fixed_decimals(2));
            });
            ui.small("Deadzone currently affects the visualization only, not game input.");
            ui.add_space(12.0);

            if let Some(error) = &self.error {
                ui.colored_label(Color32::LIGHT_RED, format!("Gamepad initialization failed: {error}"));
                return;
            }

            let Some(gilrs) = self.gilrs.as_ref() else { return; };
            let mut connected = 0;
            for (id, pad) in gilrs.gamepads() {
                if !pad.is_connected() { continue; }
                connected += 1;
                ui.group(|ui| {
                    ui.heading(pad.name());
                    ui.label(format!("Device ID: {id:?}"));
                    ui.label(format!("Battery: {}", battery_label(pad.power_info())));
                    ui.add_space(8.0);
                    ui.label("Live button tester");
                    ui.horizontal_wrapped(|ui| {
                        for (button, label) in [
                            (Button::South, "A / Cross"),
                            (Button::East, "B / Circle"),
                            (Button::West, "X / Square"),
                            (Button::North, "Y / Triangle"),
                            (Button::LeftTrigger, "LB / L1"),
                            (Button::RightTrigger, "RB / R1"),
                            (Button::Start, "Start"),
                            (Button::Select, "Select"),
                            (Button::DPadUp, "Up"),
                            (Button::DPadDown, "Down"),
                            (Button::DPadLeft, "Left"),
                            (Button::DPadRight, "Right"),
                        ] {
                            let pressed = pad.is_pressed(button);
                            ui.colored_label(
                                if pressed { Color32::LIGHT_GREEN } else { Color32::GRAY },
                                format!("{} {}", if pressed { "●" } else { "○" }, label),
                            );
                        }
                    });
                    ui.add_space(8.0);
                    ui.label("Analog axes");
                    for (axis, label) in [
                        (Axis::LeftStickX, "Left stick X"),
                        (Axis::LeftStickY, "Left stick Y"),
                        (Axis::RightStickX, "Right stick X"),
                        (Axis::RightStickY, "Right stick Y"),
                        (Axis::LeftZ, "Left trigger analog"),
                        (Axis::RightZ, "Right trigger analog"),
                    ] {
                        let raw = pad.value(axis);
                        let shown = if raw.abs() < self.deadzone { 0.0 } else { raw };
                        ui.horizontal(|ui| {
                            ui.label(format!("{label}: {shown:+.2}"));
                            ui.add(egui::ProgressBar::new(((shown + 1.0) / 2.0).clamp(0.0, 1.0)).desired_width(130.0));
                        });
                    }
                });
                ui.add_space(8.0);
            }
            if connected == 0 {
                ui.heading("No compatible gamepad detected");
                ui.label("Connect a controller in Windows, then try again.");
                ui.small("A paired Bluetooth device is not necessarily exposed as a usable gamepad.");
            }
            ui.separator();
            ui.small("No Xbox emulation or button remapping in v0.1. Battery readings depend on the device and driver.");
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "PadBridge",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(PadBridge::default()))),
    )
}
