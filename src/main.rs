#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod core;
use core::{
    mapping::Mapping,
    profiles::{Profile, Store},
    OutputBackend, UnsupportedOutput,
};
use eframe::egui::{self, Color32, RichText};
use gilrs::{Axis, Button, Gilrs, PowerInfo};
use std::time::Duration;
fn battery_label(power: PowerInfo) -> String {
    match power {
        PowerInfo::Unknown => "Unavailable".into(),
        PowerInfo::Wired => "Wired; battery unavailable".into(),
        PowerInfo::Discharging(p) => format!("{p}% - discharging (device report)"),
        PowerInfo::Charging(p) => format!("{p}% - charging (device report)"),
        PowerInfo::Charged => "Fully charged (device report)".into(),
    }
}
#[derive(PartialEq)]
enum Page {
    Dashboard,
    Tester,
    Profiles,
    Settings,
}
struct PadBridge {
    gilrs: Option<Gilrs>,
    error: Option<String>,
    store: Store,
    path: std::path::PathBuf,
    notice: Option<String>,
    page: Page,
    selected: Option<String>,
    output: UnsupportedOutput,
}
impl Default for PadBridge {
    fn default() -> Self {
        let path = Store::path();
        let (store, notice) = Store::load(&path);
        let (gilrs, error) = match Gilrs::new() {
            Ok(g) => (Some(g), None),
            Err(e) => (None, Some(e.to_string())),
        };
        Self {
            gilrs,
            error,
            store,
            path,
            notice,
            page: Page::Dashboard,
            selected: None,
            output: UnsupportedOutput,
        }
    }
}
impl PadBridge {
    fn save(&mut self) {
        match self.store.save(&self.path) {
            Ok(()) => self.notice = Some("Profiles saved".into()),
            Err(e) => self.notice = Some(format!("Could not save profiles: {e}")),
        }
    }
}
impl Drop for PadBridge {
    fn drop(&mut self) {
        self.output.stop();
    }
}
impl eframe::App for PadBridge {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(50));
        if let Some(g) = self.gilrs.as_mut() {
            while g.next_event().is_some() {}
        }
        egui::SidePanel::left("navigation")
            .resizable(false)
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.heading("PadBridge");
                ui.small("v0.2 preview");
                ui.add_space(16.0);
                for (page, label) in [
                    (Page::Dashboard, "Dashboard"),
                    (Page::Tester, "Tester"),
                    (Page::Profiles, "Profiles"),
                    (Page::Settings, "Settings"),
                ] {
                    if ui.selectable_label(self.page == page, label).clicked() {
                        self.page = page;
                    }
                }
            });
        egui::CentralPanel::default().show(ctx,|ui| {
   if let Some(n)=&self.notice { ui.colored_label(Color32::YELLOW,n); }
   if let Some(e)=&self.error { ui.colored_label(Color32::LIGHT_RED,format!("Gamepad initialization failed: {e}")); return; }
   match self.page {
    Page::Settings => { ui.heading("Settings"); ui.label("Dark theme"); ui.label("Refresh: 50 ms while open"); ui.label(format!("Profiles: {}",self.path.display())); ui.label(format!("Xbox virtual output: {}",if self.output.available() {"Available"} else {"Unavailable"})); ui.small("No driver is installed or downloaded. No telemetry or cloud connection."); },
    Page::Profiles => { ui.heading("Profiles"); let Some(key)=self.selected.clone() else { ui.label("Select a connected controller on Dashboard."); return; }; let d=self.store.device(&key); ui.label(format!("Controller: {key}")); let mut changed=false; ui.horizontal(|ui| { for i in 0..d.profiles.len() { if ui.selectable_label(d.active==i,&d.profiles[i].name).clicked() { d.active=i; changed=true; } } }); ui.horizontal(|ui| { if ui.button("Create").clicked() { d.profiles.push(Profile { name:format!("Profile {}",d.profiles.len()+1), mapping:Mapping::default() }); d.active=d.profiles.len()-1; changed=true; }
 if ui.button("Duplicate").clicked() { let mut p=d.active().clone(); p.name=format!("{} copy",p.name); d.profiles.push(p); d.active=d.profiles.len()-1; changed=true; }
 if ui.add_enabled(d.profiles.len()>1,egui::Button::new("Delete")).clicked() { d.profiles.remove(d.active); d.active=0; changed=true; }
 if ui.button("Restore defaults").clicked() { d.active_mut().mapping=Mapping::default(); changed=true; } }); ui.horizontal(|ui| { ui.label("Name"); changed |= ui.text_edit_singleline(&mut d.active_mut().name).changed(); }); let m=&mut d.active_mut().mapping; changed |= ui.add(egui::Slider::new(&mut m.deadzone,0.0..=0.35).text("Deadzone")).changed(); changed |= ui.add(egui::Slider::new(&mut m.trigger_threshold,0.0..=1.0).text("Trigger threshold")).changed(); changed |= ui.checkbox(&mut m.invert_left_y,"Invert left Y").changed(); changed |= ui.checkbox(&mut m.invert_right_y,"Invert right Y").changed(); ui.small("These settings affect the tester only. Game output remapping is unavailable."); if changed && d.valid_for_ui() { self.save(); } },
    Page::Dashboard | Page::Tester => { ui.heading(if self.page==Page::Dashboard {"Connected controllers"} else {"Controller tester"}); let Some(g)=self.gilrs.as_ref() else { return; }; let mut count=0; for (id,pad) in g.gamepads() { if !pad.is_connected() { continue; } count+=1; let key=format!("{}::{:?}",pad.name(),pad.uuid()); let selected=self.selected.as_deref()==Some(&key); ui.separator(); ui.horizontal(|ui| { ui.heading(pad.name()); if ui.button(if selected {"Selected"} else {"Select"}).clicked() { self.selected=Some(key.clone()); } }); ui.label(format!("Session ID: {id:?} · Connected and usable through gilrs")); ui.label(format!("Battery: {}",battery_label(pad.power_info()))); let profile=self.store.device(&key).active().clone(); ui.label(format!("Profile: {}",profile.name)); if self.page==Page::Tester { ui.horizontal_wrapped(|ui| { for (b,label) in [(Button::South,"A / Cross"),(Button::East,"B / Circle"),(Button::West,"X / Square"),(Button::North,"Y / Triangle"),(Button::LeftTrigger,"LB"),(Button::RightTrigger,"RB"),(Button::Start,"Start"),(Button::Select,"Select"),(Button::DPadUp,"Up"),(Button::DPadDown,"Down"),(Button::DPadLeft,"Left"),(Button::DPadRight,"Right")] { ui.colored_label(if pad.is_pressed(b) { Color32::LIGHT_GREEN } else { Color32::GRAY },format!("{} {label}",if pad.is_pressed(b) {"●"} else {"○"})); } }); for (axis,label,invert) in [(Axis::LeftStickX,"Left X",false),(Axis::LeftStickY,"Left Y",profile.mapping.invert_left_y),(Axis::RightStickX,"Right X",false),(Axis::RightStickY,"Right Y",profile.mapping.invert_right_y),(Axis::LeftZ,"Left trigger",false),(Axis::RightZ,"Right trigger",false)] { let raw=pad.value(axis); let value=profile.mapping.axis(raw,invert); ui.horizontal(|ui| { ui.label(format!("{label}: {value:+.2} (raw {raw:+.2})")); ui.add(egui::ProgressBar::new(((value+1.0)/2.0).clamp(0.0,1.0)).desired_width(130.0)); }); } ui.label(format!("Trigger threshold: left {} · right {}", profile.mapping.trigger_active(pad.value(Axis::LeftZ)), profile.mapping.trigger_active(pad.value(Axis::RightZ)))); ui.small("Profile transforms affect this display only."); } } if count==0 { ui.label(RichText::new("No compatible gamepad detected").size(20.0)); ui.label("Connect a controller in Windows. A paired Bluetooth device may not be exposed as a usable gamepad."); } }
   }
  });
    }
}
trait UiValidation {
    fn valid_for_ui(&self) -> bool;
}
impl UiValidation for core::profiles::DeviceProfiles {
    fn valid_for_ui(&self) -> bool {
        !self.active().name.trim().is_empty() && self.active().name.len() <= 80
    }
}
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "PadBridge",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(PadBridge::default()))
        }),
    )
}
