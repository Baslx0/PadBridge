use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mapping {
    pub deadzone: f32,
    pub invert_left_y: bool,
    pub invert_right_y: bool,
    pub trigger_threshold: f32,
}
impl Default for Mapping {
    fn default() -> Self {
        Self {
            deadzone: 0.1,
            invert_left_y: false,
            invert_right_y: false,
            trigger_threshold: 0.1,
        }
    }
}
impl Mapping {
    pub fn valid(&self) -> bool {
        self.deadzone.is_finite()
            && (0.0..=0.35).contains(&self.deadzone)
            && self.trigger_threshold.is_finite()
            && (0.0..=1.0).contains(&self.trigger_threshold)
    }
    pub fn axis(&self, value: f32, invert: bool) -> f32 {
        let value = value.clamp(-1.0, 1.0);
        let value = if value.abs() < self.deadzone {
            0.0
        } else {
            value
        };
        if invert {
            -value
        } else {
            value
        }
    }
    pub fn trigger_active(&self, value: f32) -> bool {
        value >= self.trigger_threshold
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transforms() {
        let m = Mapping::default();
        assert_eq!(m.axis(0.05, false), 0.0);
        assert_eq!(m.axis(0.5, true), -0.5);
        assert!(m.trigger_active(0.1));
        assert!(!Mapping {
            deadzone: f32::NAN,
            ..m
        }
        .valid());
    }
}
