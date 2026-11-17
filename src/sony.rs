//! Read-only Sony HID input fallback when Windows Gaming Input does not enumerate a pad.
//! Report offsets follow SDL's documented PS4/PS5 HID layouts; no output reports are sent.
use gilrs::Button;
use hidapi::{DeviceInfo, HidApi, HidDevice};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    pub axes: [f32; 4],
    pub triggers: [f32; 2],
    pub buttons: [bool; 14],
}
impl Input {
    pub fn pressed(&self, button: Button) -> bool {
        let index = match button {
            Button::North => 0,
            Button::West => 1,
            Button::South => 2,
            Button::East => 3,
            Button::LeftTrigger => 4,
            Button::RightTrigger => 5,
            Button::Select => 6,
            Button::Start => 7,
            Button::DPadUp => 8,
            Button::DPadDown => 9,
            Button::DPadLeft => 10,
            Button::DPadRight => 11,
            Button::Mode => 12,
            _ => return false,
        };
        self.buttons[index]
    }
}

fn axis(value: u8, invert: bool) -> f32 {
    let value = (value as f32 - 127.5) / 127.5;
    if invert {
        -value
    } else {
        value
    }
}

pub fn parse(pid: u16, data: &[u8]) -> Option<Input> {
    let (offset, button_offset, trigger_offset) = match (pid, data.first().copied()?) {
        (0x0ce6 | 0x0df2, 0x01) if data.len() == 10 || data.len() == 78 => (1, 4, 7),
        (0x0ce6 | 0x0df2, 0x01) if data.len() >= 11 => (1, 7, 4),
        (0x0ce6 | 0x0df2, 0x31) if data.len() >= 13 => (2, 7, 4),
        (0x05c4 | 0x09cc, 0x01) if data.len() >= 10 => (1, 4, 7),
        (0x05c4 | 0x09cc, 0x11..=0x19) if data.len() >= 12 => (3, 4, 7),
        _ => return None,
    };
    let bytes = data.get(offset..)?;
    let b0 = *bytes.get(button_offset)?;
    let b1 = *bytes.get(button_offset + 1)?;
    let b2 = *bytes.get(button_offset + 2)?;
    let hat = b0 & 0x0f;
    let buttons = [
        b0 & 0x80 != 0,
        b0 & 0x10 != 0,
        b0 & 0x20 != 0,
        b0 & 0x40 != 0,
        b1 & 0x01 != 0,
        b1 & 0x02 != 0,
        b1 & 0x10 != 0,
        b1 & 0x20 != 0,
        matches!(hat, 0 | 1 | 7),
        matches!(hat, 3..=5),
        matches!(hat, 5..=7),
        matches!(hat, 1..=3),
        b2 & 0x01 != 0,
        b2 & 0x02 != 0,
    ];
    Some(Input {
        axes: [
            axis(*bytes.first()?, false),
            axis(*bytes.get(1)?, true),
            axis(*bytes.get(2)?, false),
            axis(*bytes.get(3)?, true),
        ],
        triggers: [
            *bytes.get(trigger_offset)? as f32 / 255.0,
            *bytes.get(trigger_offset + 1)? as f32 / 255.0,
        ],
        buttons,
    })
}

fn supported(info: &DeviceInfo) -> bool {
    info.vendor_id() == 0x054c
        && matches!(info.product_id(), 0x0ce6 | 0x0df2 | 0x05c4 | 0x09cc)
        && info.usage_page() == 1
        && matches!(info.usage(), 4 | 5)
}

#[derive(Clone)]
pub struct Device {
    pub key: String,
    pub name: String,
    pub connection: String,
    pub input: Input,
    pub last_report: Instant,
}

enum Event {
    Report(Device),
    Removed(String),
    Error(String),
}
struct OpenDevice {
    key: String,
    name: String,
    connection: String,
    pid: u16,
    device: HidDevice,
    last_emit: Instant,
}

type OutputRoute = Arc<Mutex<Option<(String, Sender<Input>)>>>;

fn worker(rx_stop: Receiver<()>, tx: Sender<Event>, output: OutputRoute) {
    let mut api = match HidApi::new() {
        Ok(api) => api,
        Err(e) => {
            let _ = tx.send(Event::Error(format!("Sony HID initialization failed: {e}")));
            return;
        }
    };
    let mut open: HashMap<String, OpenDevice> = HashMap::new();
    let mut last_scan = Instant::now() - Duration::from_secs(2);
    let mut data = [0u8; 128];
    loop {
        if rx_stop.try_recv().is_ok() {
            break;
        }
        if last_scan.elapsed() >= Duration::from_secs(2) {
            if let Err(e) = api.refresh_devices() {
                let _ = tx.send(Event::Error(format!("Sony HID scan failed: {e}")));
            }
            let infos: Vec<_> = api
                .device_list()
                .filter(|d| supported(d))
                .cloned()
                .collect();
            let paths: HashSet<_> = infos
                .iter()
                .map(|i| i.path().to_string_lossy().into_owned())
                .collect();
            let lost: Vec<_> = open
                .keys()
                .filter(|k| !paths.contains(*k))
                .cloned()
                .collect();
            for path in lost {
                if let Some(d) = open.remove(&path) {
                    let _ = tx.send(Event::Removed(d.key));
                }
            }
            for info in infos {
                let path = info.path().to_string_lossy().into_owned();
                if open.contains_key(&path) {
                    continue;
                }
                if let Ok(device) = info.open_device(&api) {
                    let name = info
                        .product_string()
                        .unwrap_or("PlayStation controller")
                        .to_string();
                    let key = format!(
                        "{}::{:04x}::{}",
                        name,
                        info.product_id(),
                        info.serial_number().unwrap_or(&path)
                    );
                    open.insert(
                        path,
                        OpenDevice {
                            key,
                            name,
                            connection: format!("{:?}", info.bus_type()),
                            pid: info.product_id(),
                            device,
                            last_emit: Instant::now() - Duration::from_secs(1),
                        },
                    );
                }
            }
            last_scan = Instant::now();
        }
        if open.is_empty() {
            thread::sleep(Duration::from_millis(100));
            continue;
        }
        let mut lost = Vec::new();
        for (path, d) in &mut open {
            match d.device.read_timeout(&mut data, 20) {
                Ok(n) if n > 0 => {
                    if let Some(input) = parse(d.pid, &data[..n]) {
                        if d.last_emit.elapsed() >= Duration::from_millis(40) {
                            if let Ok(route) = output.lock() {
                                if let Some((key, sender)) = route.as_ref() {
                                    if *key == d.key {
                                        let _ = sender.send(input.clone());
                                    }
                                }
                            }
                            let _ = tx.send(Event::Report(Device {
                                key: d.key.clone(),
                                name: d.name.clone(),
                                connection: d.connection.clone(),
                                input,
                                last_report: Instant::now(),
                            }));
                            d.last_emit = Instant::now();
                        }
                    }
                }
                Ok(_) => {}
                Err(_) => lost.push(path.clone()),
            }
        }
        for path in lost {
            if let Some(d) = open.remove(&path) {
                let _ = tx.send(Event::Removed(d.key));
            }
        }
    }
}

pub struct SonyBackend {
    rx: Receiver<Event>,
    stop: Sender<()>,
    join: Option<JoinHandle<()>>,
    pub devices: HashMap<String, Device>,
    pub error: Option<String>,
    output: OutputRoute,
}
impl SonyBackend {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let (stop, rx_stop) = mpsc::channel();
        let output: OutputRoute = Arc::new(Mutex::new(None));
        let worker_output = output.clone();
        let join = thread::Builder::new()
            .name("sony-hid".into())
            .spawn(move || worker(rx_stop, tx, worker_output))
            .ok();
        Self {
            rx,
            stop,
            join,
            devices: HashMap::new(),
            error: None,
            output,
        }
    }
    pub fn set_output(&mut self, route: Option<(String, Sender<Input>)>) {
        if let Ok(mut output) = self.output.lock() {
            *output = route;
        }
    }
    pub fn poll(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Event::Report(device) => {
                    self.devices.insert(device.key.clone(), device);
                }
                Event::Removed(key) => {
                    self.devices.remove(&key);
                }
                Event::Error(error) => self.error = Some(error),
            }
        }
        self.devices
            .retain(|_, d| d.last_report.elapsed() < Duration::from_secs(2));
    }
}
impl Drop for SonyBackend {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dualsense_simple_report() {
        let bytes = [1, 128, 128, 128, 128, 0x20, 0x11, 0, 128, 64];
        let state = parse(0x0ce6, &bytes).unwrap();
        assert!(state.pressed(Button::South));
        assert!(state.pressed(Button::LeftTrigger));
        assert!(state.pressed(Button::Select));
        assert!(state.pressed(Button::DPadUp));
        assert!((state.triggers[0] - 128.0 / 255.0).abs() < 0.001);
    }
    #[test]
    fn dualshock_bluetooth_report() {
        let bytes = [0x11, 0, 0, 128, 128, 128, 128, 0x42, 0x02, 0, 0, 255];
        let state = parse(0x09cc, &bytes).unwrap();
        assert!(state.pressed(Button::East));
        assert!(state.pressed(Button::RightTrigger));
        assert!(state.pressed(Button::DPadRight));
        assert_eq!(state.triggers[1], 1.0);
    }
    #[test]
    fn dualsense_extended_report() {
        let mut bytes = [0u8; 64];
        bytes[0] = 1;
        bytes[1] = 128;
        bytes[2] = 128;
        bytes[3] = 128;
        bytes[4] = 128;
        bytes[5] = 255;
        bytes[8] = 0x20;
        let state = parse(0x0ce6, &bytes).unwrap();
        assert_eq!(state.triggers[0], 1.0);
        assert!(state.pressed(Button::South));
    }
    #[test]
    fn dualshock_usb_report() {
        let bytes = [1, 128, 128, 128, 128, 0x10, 0x01, 0, 0, 0];
        let state = parse(0x05c4, &bytes).unwrap();
        assert!(state.pressed(Button::West));
        assert!(state.pressed(Button::LeftTrigger));
    }
    #[test]
    fn dualsense_usb_left_controls() {
        let mut bytes = [0u8; 64];
        bytes[0] = 1;
        bytes[5] = 255; // L2 analog
        bytes[8] = 0x00; // D-pad up
        bytes[9] = 0x01; // L1
        let state = parse(0x0ce6, &bytes).unwrap();
        assert!(state.pressed(Button::LeftTrigger));
        assert!(state.pressed(Button::DPadUp));
        assert_eq!(state.triggers[0], 1.0);
    }
    #[test]
    fn touchpad_and_dpad() {
        let bytes = [1, 128, 128, 128, 128, 0x00, 0x00, 0x02, 0, 0];
        let state = parse(0x0ce6, &bytes).unwrap();
        assert!(state.buttons[13]);
        assert!(state.pressed(Button::DPadUp));
    }
    #[test]
    fn unsupported_report_rejected() {
        assert!(parse(0x0ce6, &[0x99, 0, 0]).is_none());
    }
}
