use crate::sony::Input;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};
use vigem_client::{Client, TargetId, XButtons, XGamepad, Xbox360Wired};

const DRIVER_URL: &str = "https://github.com/nefarius/ViGEmBus/releases/download/v1.22.0/ViGEmBus_1.22.0_x64_x86_arm64.exe";
const DRIVER_NAME: &str = "ViGEmBus_1.22.0_x64_x86_arm64.exe";
const DRIVER_SHA256: &str = "89220a7865076b342892f98865f3499fb7c4cfd673159e89d352c360fd014c6a";
const MAX_DRIVER_BYTES: u64 = 16 * 1024 * 1024;

pub struct XboxOutput {
    target: Option<Xbox360Wired<Client>>,
    pub error: Option<String>,
    download: Option<Receiver<Result<PathBuf, String>>>,
    pub installer: Option<PathBuf>,
}

impl XboxOutput {
    pub fn new() -> Self {
        Self {
            target: None,
            error: None,
            download: None,
            installer: None,
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
            Err(vigem_client::Error::BusNotFound) => {
                self.error = Some(
                    "ViGEmBus is missing. Downloading the verified installer from Nefarius.".into(),
                );
                self.download_driver();
            }
            Err(error) => {
                self.error = Some(format!("Virtual Xbox controller unavailable: {error:?}"))
            }
        }
    }

    pub fn stop(&mut self) {
        self.target = None;
    }

    pub fn downloading(&self) -> bool {
        self.download.is_some()
    }

    pub fn poll_download(&mut self) {
        let Some(receiver) = &self.download else {
            return;
        };
        if let Ok(result) = receiver.try_recv() {
            self.download = None;
            match result {
                Ok(path) => {
                    self.error = Some("Verified ViGEmBus installer downloaded. Open it to install the driver, then retry Xbox output.".into());
                    self.installer = Some(path);
                }
                Err(error) => self.error = Some(format!("Driver download failed: {error}")),
            }
        }
    }

    fn download_driver(&mut self) {
        if self.downloading() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.download = Some(rx);
        thread::spawn(move || {
            let _ = tx.send(fetch_driver());
        });
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

fn verified(bytes: &[u8]) -> bool {
    format!("{:x}", Sha256::digest(bytes)) == DRIVER_SHA256
}

fn fetch_driver() -> Result<PathBuf, String> {
    let base = std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is unavailable")?;
    let directory = PathBuf::from(base).join("PadBridge").join("Drivers");
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let path = directory.join(DRIVER_NAME);
    if let Ok(bytes) = fs::read(&path) {
        if verified(&bytes) {
            return Ok(path);
        }
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;
    let response = client
        .get(DRIVER_URL)
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    if response
        .content_length()
        .is_some_and(|size| size > MAX_DRIVER_BYTES)
    {
        return Err("Installer exceeds maximum expected size".into());
    }
    let mut bytes = Vec::new();
    response
        .take(MAX_DRIVER_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_DRIVER_BYTES || !verified(&bytes) {
        return Err("Installer SHA-256 did not match the pinned official release".into());
    }
    let temporary = directory.join(format!("{DRIVER_NAME}.partial"));
    fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
    fs::rename(&temporary, &path).map_err(|e| e.to_string())?;
    Ok(path)
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
