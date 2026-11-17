use crate::core::mapping::Mapping;
use eframe::egui::{self, Align2, Color32, FontId, Pos2, RichText, Stroke, Vec2};
use gilrs::{Button, Gamepad};

pub const BG: Color32 = Color32::from_rgb(17, 21, 29);
pub const PANEL: Color32 = Color32::from_rgb(25, 31, 41);
pub const MUTED: Color32 = Color32::from_rgb(147, 159, 178);
pub const ACCENT: Color32 = Color32::from_rgb(103, 182, 255);
pub const GREEN: Color32 = Color32::from_rgb(94, 214, 151);

pub fn install(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = PANEL;
    visuals.widgets.inactive.bg_fill = PANEL;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(39, 49, 64);
    visuals.selection.bg_fill = Color32::from_rgb(34, 80, 121);
    ctx.set_visuals(visuals);
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(10.0, 9.0);
    style.spacing.button_padding = Vec2::new(12.0, 8.0);
    ctx.set_style(style);
}

pub fn heading(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.label(RichText::new(title).size(27.0).strong());
    ui.label(RichText::new(subtitle).color(MUTED));
    ui.add_space(18.0);
}

pub fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(12.0);
    ui.label(RichText::new(title).size(17.0).strong());
    ui.add_space(5.0);
}

pub fn surface<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::none()
        .fill(PANEL)
        .rounding(10.0)
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add(ui)
        })
        .inner
}

pub fn stick(ui: &mut egui::Ui, label: &str, x: f32, y: f32, mapping: &Mapping, invert_y: bool) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).strong());
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(150.0), egui::Sense::hover());
        let center = rect.center();
        let radius = 58.0;
        let painter = ui.painter();
        painter.circle_stroke(center, radius, Stroke::new(1.5, MUTED));
        painter.circle_stroke(
            center,
            radius * mapping.deadzone,
            Stroke::new(1.0, Color32::from_rgb(90, 106, 128)),
        );
        painter.line_segment(
            [
                Pos2::new(center.x - radius, center.y),
                Pos2::new(center.x + radius, center.y),
            ],
            Stroke::new(1.0, Color32::from_rgb(61, 72, 89)),
        );
        painter.line_segment(
            [
                Pos2::new(center.x, center.y - radius),
                Pos2::new(center.x, center.y + radius),
            ],
            Stroke::new(1.0, Color32::from_rgb(61, 72, 89)),
        );
        let px = center.x + mapping.axis(x, false) * radius;
        let py = center.y - mapping.axis(y, invert_y) * radius;
        painter.circle_filled(Pos2::new(px, py), 8.0, ACCENT);
        ui.label(
            RichText::new(format!(
                "X {:+.2}   Y {:+.2}",
                mapping.axis(x, false),
                mapping.axis(y, invert_y)
            ))
            .monospace()
            .color(MUTED),
        );
    });
}

pub fn button_grid(ui: &mut egui::Ui, pad: &Gamepad<'_>) {
    for row in [
        [
            (Button::North, "Triangle / Y"),
            (Button::West, "Square / X"),
            (Button::South, "Cross / A"),
            (Button::East, "Circle / B"),
        ],
        [
            (Button::LeftTrigger, "L1 / LB"),
            (Button::RightTrigger, "R1 / RB"),
            (Button::Select, "Select"),
            (Button::Start, "Start"),
        ],
        [
            (Button::DPadUp, "D-pad up"),
            (Button::DPadDown, "D-pad down"),
            (Button::DPadLeft, "D-pad left"),
            (Button::DPadRight, "D-pad right"),
        ],
    ] {
        ui.horizontal_wrapped(|ui| {
            for (button, label) in row {
                let active = pad.is_pressed(button);
                egui::Frame::none()
                    .fill(if active {
                        Color32::from_rgb(33, 78, 61)
                    } else {
                        Color32::from_rgb(33, 40, 52)
                    })
                    .rounding(6.0)
                    .inner_margin(egui::Margin::symmetric(10.0, 7.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("{} {label}", if active { "●" } else { "○" }))
                                .color(if active { GREEN } else { MUTED }),
                        );
                    });
            }
        });
    }
}

pub fn trigger(ui: &mut egui::Ui, label: &str, value: f32, mapping: &Mapping) {
    let normalized = value.clamp(0.0, 1.0);
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            egui::ProgressBar::new(normalized)
                .desired_width(180.0)
                .show_percentage(),
        );
        ui.label(
            RichText::new(if mapping.trigger_active(normalized) {
                "Active"
            } else {
                "Idle"
            })
            .color(if mapping.trigger_active(normalized) {
                GREEN
            } else {
                MUTED
            }),
        );
    });
}

pub fn empty(ui: &mut egui::Ui, title: &str, detail: &str) {
    ui.add_space(((ui.available_height() - 160.0) / 2.0).max(24.0));
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 130.0), egui::Sense::hover());
    let p = ui.painter();
    p.text(
        Pos2::new(rect.center().x, rect.center().y - 15.0),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(21.0),
        Color32::WHITE,
    );
    p.text(
        Pos2::new(rect.center().x, rect.center().y + 17.0),
        Align2::CENTER_CENTER,
        detail,
        FontId::proportional(14.0),
        MUTED,
    );
}
