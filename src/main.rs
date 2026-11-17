#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod core;
mod output;
mod sony;
mod ui;

use core::{
    mapping::Mapping,
    profiles::{Profile, Store},
};
use eframe::egui::{self, Color32, RichText};
use gilrs::{Axis, Gilrs, PowerInfo};
use output::XboxOutput;
use sony::SonyBackend;
use std::{path::PathBuf, time::Duration};

fn power_label(power: PowerInfo) -> String {
    match power {
        PowerInfo::Unknown => "Battery unavailable".into(),
        PowerInfo::Wired => "Wired · battery unavailable".into(),
        PowerInfo::Discharging(p) => format!("{p}% · discharging"),
        PowerInfo::Charging(p) => format!("{p}% · charging"),
        PowerInfo::Charged => "Fully charged".into(),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Dashboard,
    Tester,
    Profiles,
    Settings,
}
impl Page {
    fn title(self) -> &'static str {
        match self {
            Self::Dashboard => "Controllers",
            Self::Tester => "Input tester",
            Self::Profiles => "Profiles",
            Self::Settings => "About & settings",
        }
    }
    fn subtitle(self) -> &'static str {
        match self {
            Self::Dashboard => "Connected gamepads and their current status",
            Self::Tester => "Live input from the selected controller",
            Self::Profiles => "Local configurations for the selected controller",
            Self::Settings => "Application status and storage",
        }
    }
}

struct App {
    gilrs: Option<Gilrs>,
    sony: SonyBackend,
    error: Option<String>,
    store: Store,
    path: PathBuf,
    notice: Option<String>,
    page: Page,
    selected: Option<String>,
    output: XboxOutput,
}
impl Default for App {
    fn default() -> Self {
        let path = Store::path();
        let (store, notice) = Store::load(&path);
        let (gilrs, error) = match Gilrs::new() {
            Ok(g) => (Some(g), None),
            Err(e) => (None, Some(e.to_string())),
        };
        Self {
            gilrs,
            sony: SonyBackend::new(),
            error,
            store,
            path,
            notice,
            page: Page::Dashboard,
            selected: None,
            output: XboxOutput::new(),
        }
    }
}
impl App {
    fn save(&mut self) {
        self.notice = Some(match self.store.save(&self.path) {
            Ok(()) => "Profile saved".into(),
            Err(e) => format!("Could not save profile: {e}"),
        });
    }
    fn connected(&self) -> Vec<String> {
        let mut keys = Vec::new();
        let mut gilrs_names = std::collections::HashMap::<String, usize>::new();
        if let Some(g) = self.gilrs.as_ref() {
            for (_, pad) in g.gamepads().filter(|(_, pad)| pad.is_connected()) {
                keys.push(format!("{}::{:?}", pad.name(), pad.uuid()));
                *gilrs_names.entry(pad.name().to_string()).or_default() += 1;
            }
        }
        let mut sony_devices: Vec<_> = self.sony.devices.values().collect();
        sony_devices.sort_by(|a, b| a.key.cmp(&b.key));
        for device in sony_devices {
            let matched = gilrs_names.entry(device.name.clone()).or_default();
            if *matched > 0 {
                *matched -= 1;
            } else {
                keys.push(device.key.clone());
            }
        }
        keys.sort();
        keys
    }
    fn current_key(&self, keys: &[String]) -> Option<String> {
        self.selected
            .as_ref()
            .filter(|s| keys.contains(s))
            .cloned()
            .or_else(|| keys.first().cloned())
    }
    fn navigation(&mut self, ctx: &egui::Context, count: usize) {
        egui::SidePanel::left("navigation")
            .resizable(false)
            .exact_width(184.0)
            .frame(egui::Frame::none().fill(ui::PANEL).inner_margin(18.0))
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(RichText::new("PadBridge").size(23.0).strong());
                ui.label(
                    RichText::new("CONTROLLER MANAGER")
                        .size(10.0)
                        .color(ui::MUTED),
                );
                ui.add_space(30.0);
                for (page, label) in [
                    (Page::Dashboard, "Controllers"),
                    (Page::Tester, "Input tester"),
                    (Page::Profiles, "Profiles"),
                    (Page::Settings, "About & settings"),
                ] {
                    let active = self.page == page;
                    if ui
                        .add_sized([148.0, 36.0], egui::SelectableLabel::new(active, label))
                        .clicked()
                    {
                        self.page = page;
                    }
                }
                ui.add_space(18.0);
                ui.separator();
                ui.label(
                    RichText::new(format!("{count} connected")).color(if count > 0 {
                        ui::GREEN
                    } else {
                        ui::MUTED
                    }),
                );
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(RichText::new("v0.2 · local only").small().color(ui::MUTED));
                });
            });
    }
    fn dashboard(&mut self, ui: &mut egui::Ui, keys: &[String]) {
        ui::surface(ui, |ui| {
            ui.label(
                RichText::new(if self.output.active() {
                    "Virtual Xbox controller active"
                } else if self.output.busy() {
                    "Starting virtual Xbox controller..."
                } else {
                    "Virtual Xbox controller off"
                })
                .strong()
                .color(if self.output.active() {
                    ui::GREEN
                } else {
                    Color32::YELLOW
                }),
            );
            ui.label(RichText::new("Select a Sony controller, then enable Xbox output for games that need XInput. VIIPER and USB/IP run locally.").color(ui::MUTED));
            if self.output.busy() {
                if let Some(slot) = self.output.slot {
                    ui.label(format!("XInput slot {slot}"));
                }
                if ui.button("Stop Xbox output").clicked() {
                    self.sony.set_output(None);
                    self.output.stop();
                }
            } else if self
                .current_key(keys)
                .is_some_and(|key| self.sony.devices.contains_key(&key))
                && ui.button("Start Xbox output").clicked()
            {
                let key = self.current_key(keys).expect("selected Sony key exists");
                if let Some(sender) = self.output.start() {
                    self.sony.set_output(Some((key, sender)));
                }
            }
            if let Some(error) = &self.output.error {
                ui.colored_label(Color32::LIGHT_RED, error);
            }
            if self.output.downloading() || (self.output.busy() && !self.output.active()) {
                ui.spinner();
            }
            if let Some(path) = &self.output.installer {
                ui.label(RichText::new("Driver installation can briefly restart USB devices. Save active work first.").color(ui::MUTED));
                if ui.button("Open verified driver installer").clicked() {
                    if let Err(error) = std::process::Command::new(path).spawn() {
                        self.output.error = Some(format!("Could not open installer: {error}"));
                    }
                }
            }
        });
        ui.add_space(10.0);
        if keys.is_empty() {
            ui::empty(
                ui,
                "No controller connected",
                "Connect a USB or Bluetooth gamepad supported by Windows.",
            );
            ui.label(
                RichText::new("Sony HID and gilrs devices appear while usable input is available.")
                    .color(ui::MUTED),
            );
            return;
        }
        let current = self.current_key(keys);
        ui::section(ui, "Devices");
        if let Some(g) = self.gilrs.as_ref() {
            for (id, pad) in g.gamepads() {
                if !pad.is_connected() {
                    continue;
                }
                let key = format!("{}::{:?}", pad.name(), pad.uuid());
                let selected = current.as_ref() == Some(&key);
                ui::surface(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(pad.name()).size(18.0).strong());
                            ui.label(
                                RichText::new(format!("Connected · gilrs ID {id:?}"))
                                    .color(ui::GREEN),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(if selected { "Viewing" } else { "View" })
                                .clicked()
                            {
                                self.selected = Some(key.clone());
                            }
                        });
                    });
                    ui.add_space(5.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(power_label(pad.power_info())).color(ui::MUTED));
                        ui.label(
                            RichText::new(format!(
                                "Profile: {}",
                                self.store.device(&key).active().name
                            ))
                            .color(ui::MUTED),
                        );
                    });
                });
                ui.add_space(9.0);
            }
        }
        for device in self
            .sony
            .devices
            .values()
            .filter(|device| keys.contains(&device.key))
        {
            let key = device.key.clone();
            let selected = current.as_ref() == Some(&key);
            ui::surface(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&device.name).size(18.0).strong());
                        ui.label(
                            RichText::new(format!("Connected · Sony HID · {}", device.connection))
                                .color(ui::GREEN),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(if selected { "Viewing" } else { "View" })
                            .clicked()
                        {
                            self.selected = Some(key.clone());
                        }
                    });
                });
                ui.add_space(5.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Battery unavailable").color(ui::MUTED));
                    ui.label(
                        RichText::new(format!(
                            "Profile: {}",
                            self.store.device(&key).active().name
                        ))
                        .color(ui::MUTED),
                    );
                });
            });
            ui.add_space(9.0);
        }
        ui.add_space(10.0);
        ui::section(ui, "Selected device");
        if let Some(key) = current {
            let name = key.split("::").next().unwrap_or("Controller");
            ui::surface(ui, |ui| {
                ui.label(RichText::new(name).size(19.0).strong());
                ui.label(
                    RichText::new(
                        "Input is available for testing. Profiles change the tester display only.",
                    )
                    .color(ui::MUTED),
                );
                ui.add_space(9.0);
                ui.horizontal(|ui| {
                    if ui.button("Open tester").clicked() {
                        self.page = Page::Tester;
                    }
                    if ui.button("Edit profile").clicked() {
                        self.page = Page::Profiles;
                    }
                });
            });
        }
    }
    fn tester(&mut self, ui: &mut egui::Ui, keys: &[String]) {
        let Some(key) = self.current_key(keys) else {
            ui::empty(
                ui,
                "No input to test",
                "Connect a compatible controller to begin.",
            );
            return;
        };
        let profile = self.store.device(&key).active().clone();
        if let Some(device) = self.sony.devices.get(&key) {
            let input = &device.input;
            ui::tester_panel(
                ui,
                &device.name,
                &profile.name,
                &profile.mapping,
                input.axes,
                input.triggers,
                (
                    |button| input.pressed(button),
                    &[("PS", input.buttons[12]), ("Touchpad", input.buttons[13])],
                ),
            );
            return;
        }
        let Some(g) = self.gilrs.as_ref() else {
            return;
        };
        let Some((_, pad)) = g
            .gamepads()
            .find(|(_, p)| p.is_connected() && format!("{}::{:?}", p.name(), p.uuid()) == key)
        else {
            return;
        };
        ui::tester_panel(
            ui,
            pad.name(),
            &profile.name,
            &profile.mapping,
            [
                pad.value(Axis::LeftStickX),
                pad.value(Axis::LeftStickY),
                pad.value(Axis::RightStickX),
                pad.value(Axis::RightStickY),
            ],
            [pad.value(Axis::LeftZ), pad.value(Axis::RightZ)],
            (|button| pad.is_pressed(button), &[]),
        );
    }
    fn profiles(&mut self, ui: &mut egui::Ui, keys: &[String]) {
        let Some(key) = self.current_key(keys) else {
            ui::empty(
                ui,
                "No controller selected",
                "Connect a controller to manage its profiles.",
            );
            return;
        };
        let mut changed = false;
        let mut valid = true;
        let device = self.store.device(&key);
        ui::surface(ui, |ui| {
            ui.label(
                RichText::new(key.split("::").next().unwrap_or("Controller"))
                    .size(19.0)
                    .strong(),
            );
            ui.label(
                RichText::new("Profiles are saved locally and affect the tester display.")
                    .color(ui::MUTED),
            );
        });
        ui::section(ui, "Saved profiles");
        ui.horizontal_wrapped(|ui| {
            for index in 0..device.profiles.len() {
                if ui
                    .selectable_label(device.active == index, &device.profiles[index].name)
                    .clicked()
                {
                    device.active = index;
                    changed = true;
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("New profile").clicked() {
                device.profiles.push(Profile {
                    name: format!("Profile {}", device.profiles.len() + 1),
                    mapping: Mapping::default(),
                });
                device.active = device.profiles.len() - 1;
                changed = true;
            }
            if ui.button("Duplicate").clicked() {
                let mut copy = device.active().clone();
                copy.name = format!("{} copy", copy.name);
                device.profiles.push(copy);
                device.active = device.profiles.len() - 1;
                changed = true;
            }
            if ui
                .add_enabled(device.profiles.len() > 1, egui::Button::new("Delete"))
                .clicked()
            {
                device.profiles.remove(device.active);
                device.active = 0;
                changed = true;
            }
        });
        ui::section(ui, "Configuration");
        ui::surface(ui, |ui| {
            let profile = device.active_mut();
            ui.horizontal(|ui| {
                ui.label("Name");
                changed |= ui.text_edit_singleline(&mut profile.name).changed();
            });
            valid = !profile.name.trim().is_empty() && profile.name.len() <= 80;
            if !valid {
                ui.colored_label(Color32::LIGHT_RED, "Enter a name of 1–80 characters.");
            }
            ui.add_space(5.0);
            changed |= ui
                .add(
                    egui::Slider::new(&mut profile.mapping.deadzone, 0.0..=0.35)
                        .text("Stick deadzone"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut profile.mapping.trigger_threshold, 0.0..=1.0)
                        .text("Trigger threshold"),
                )
                .changed();
            changed |= ui
                .checkbox(&mut profile.mapping.invert_left_y, "Invert left stick Y")
                .changed();
            changed |= ui
                .checkbox(&mut profile.mapping.invert_right_y, "Invert right stick Y")
                .changed();
            if ui.button("Restore defaults").clicked() {
                profile.mapping = Mapping::default();
                changed = true;
            }
        });
        if changed && valid && device.valid() {
            self.save();
        }
        ui.add_space(12.0);
        ui.label(
            RichText::new(
                "Profile mapping affects the tester only; Xbox output uses raw Sony input.",
            )
            .color(ui::MUTED),
        );
    }
    fn settings(&mut self, ui: &mut egui::Ui) {
        ui::section(ui, "Application");
        ui::surface(ui, |ui| {
            ui.label("PadBridge 0.2.0");
            ui.label("Dark theme · 50 ms UI refresh");
            ui.label("Local controller input through gilrs and Sony HID");
        });
        ui::section(ui, "Storage");
        ui::surface(ui, |ui| {
            ui.label("Profiles file");
            ui.label(
                RichText::new(self.path.display().to_string())
                    .monospace()
                    .color(ui::MUTED),
            );
        });
        ui::section(ui, "Game compatibility");
        ui::surface(ui, |ui| {
            ui.label(
                "Virtual Xbox output uses VIIPER and a USB/IP driver for Sony HID controllers.",
            );
            ui.label(RichText::new("Select the Sony controller on the Controllers page and start Xbox output. Steam Input remains another option.").color(ui::MUTED));
        });
        ui::section(ui, "Output");
        ui::surface(ui, |ui| {
            ui.label(format!(
                "Virtual Xbox controller: {}",
                if self.output.active() {
                    "Active"
                } else if self.output.busy() {
                    "Starting"
                } else {
                    "Off"
                }
            ));
            ui.label(
                RichText::new("If USB/IP is missing, PadBridge prepares its bundled installer. Installation requires your action.")
                    .color(ui::MUTED),
            );
        });
    }
}
impl Drop for App {
    fn drop(&mut self) {
        self.sony.set_output(None);
        self.output.stop();
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(50));
        if let Some(g) = self.gilrs.as_mut() {
            while g.next_event().is_some() {}
        }
        self.sony.poll();
        self.output.poll();
        if !self.output.busy() {
            self.sony.set_output(None);
        }
        let keys = self.connected();
        self.navigation(ctx, keys.len());
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(ui::BG)
                    .inner_margin(egui::Margin::symmetric(28.0, 20.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui::heading(ui, self.page.title(), self.page.subtitle());
                        if let Some(error) = &self.error {
                            ui.colored_label(
                                Color32::LIGHT_RED,
                                format!("Gamepad initialization failed: {error}"),
                            );
                            return;
                        }
                        if let Some(error) = &self.sony.error {
                            ui.colored_label(Color32::LIGHT_RED, error);
                        }
                        if let Some(notice) = &self.notice {
                            ui.colored_label(ui::MUTED, notice);
                        }
                        match self.page {
                            Page::Dashboard => self.dashboard(ui, &keys),
                            Page::Tester => self.tester(ui, &keys),
                            Page::Profiles => self.profiles(ui, &keys),
                            Page::Settings => self.settings(ui),
                        }
                    });
            });
    }
}
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "PadBridge",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([940.0, 700.0])
                .with_min_inner_size([650.0, 500.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            ui::install(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}
