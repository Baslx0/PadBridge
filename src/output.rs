use crate::sony::Input;
use vigem_client::{Client, TargetId, XButtons, XGamepad, Xbox360Wired};

pub struct XboxOutput {
    target: Option<Xbox360Wired<Client>>,
    pub error: Option<String>,
}

impl XboxOutput {
    pub fn new() -> Self {
        Self {
            target: None,
            error: None,
        }
    }

    pub fn active(&self) -> bool {
        self.target.is_some()
    }

    pub fn start(&mut self) {
        if self.active() {
            return;
        }
        self.error = None;
        let result = (|| {
            let client = Client::connect()?;
            let mut target = Xbox360Wired::new(client, TargetId::XBOX360_WIRED);
            target.plugin()?;
            target.wait_ready()?;
            Ok::<_, vigem_client::Error>(target)
        })();
        match result {
            Ok(target) => self.target = Some(target),
            Err(error) => self.error = Some(format!("Virtual Xbox controller unavailable: {error:?}. Install ViGEmBus from the official Nefarius release.")),
        }
    }

    pub fn stop(&mut self) {
        self.target = None;
    }

    pub fn update(&mut self, input: &Input) {
        if let Some(target) = self.target.as_mut() {
            if let Err(error) = target.update(&report(input)) {
                self.error = Some(format!("Virtual Xbox update failed: {error:?}"));
                self.stop();
            }
        }
    }
}

fn stick(value: f32) -> i16 {
    (value.clamp(-1.0, 1.0) * if value < 0.0 { 32768.0 } else { 32767.0 }) as i16
}

fn report(input: &Input) -> XGamepad {
    let mut buttons = 0;
    for (index, flag) in [
        (0, XButtons::Y),
        (1, XButtons::X),
        (2, XButtons::A),
        (3, XButtons::B),
        (4, XButtons::LB),
        (5, XButtons::RB),
        (6, XButtons::BACK),
        (7, XButtons::START),
        (8, XButtons::UP),
        (9, XButtons::DOWN),
        (10, XButtons::LEFT),
        (11, XButtons::RIGHT),
        (12, XButtons::GUIDE),
    ] {
        if input.buttons[index] {
            buttons |= flag;
        }
    }
    XGamepad {
        buttons: XButtons(buttons),
        left_trigger: (input.triggers[0].clamp(0.0, 1.0) * 255.0) as u8,
        right_trigger: (input.triggers[1].clamp(0.0, 1.0) * 255.0) as u8,
        thumb_lx: stick(input.axes[0]),
        thumb_ly: stick(input.axes[1]),
        thumb_rx: stick(input.axes[2]),
        thumb_ry: stick(input.axes[3]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_sony_controls_to_xbox_report() {
        let mut input = Input {
            axes: [-1.0, 1.0, 0.5, -0.5],
            triggers: [1.0, 0.25],
            buttons: [false; 14],
        };
        input.buttons[2] = true; // Cross -> A
        input.buttons[4] = true; // L1 -> LB
        input.buttons[8] = true; // D-pad up
        input.buttons[13] = true; // Touchpad has no Xbox equivalent
        let xbox = report(&input);
        assert_eq!(xbox.buttons.raw, XButtons::A | XButtons::LB | XButtons::UP);
        assert_eq!(xbox.left_trigger, 255);
        assert_eq!(xbox.right_trigger, 63);
        assert_eq!(xbox.thumb_lx, i16::MIN);
        assert_eq!(xbox.thumb_ly, i16::MAX);
        assert_eq!(xbox.thumb_rx, 16383);
        assert_eq!(xbox.thumb_ry, -16384);
    }
}
