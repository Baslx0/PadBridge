use crate::sony::Input;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};
use vigem_client::{Client, TargetId, XButtons, XGamepad, Xbox360Wired};

const DRIVER_NAME: &str = "ViGEmBus_1.22.0_x64_x86_arm64.exe";
const DRIVER_SHA256: &str = "89220a7865076b342892f98865f3499fb7c4cfd673159e89d352c360fd014c6a";
const DRIVER: &[u8] = include_bytes!("../third_party/ViGEmBus/ViGEmBus_1.22.0_x64_x86_arm64.exe");

pub struct XboxOutput {
    worker: Option<JoinHandle<()>>,
    stop: Option<Sender<()>>,
    status: Option<Receiver<String>>,
    pub error: Option<String>,
    download: Option<Receiver<Result<PathBuf, String>>>,
    pub installer: Option<PathBuf>,
    pub slot: Option<u32>,
}

impl XboxOutput {
    pub fn new() -> Self {
        Self {
            worker: None,
            stop: None,
            status: None,
            error: None,
            download: None,
            installer: None,
            slot: None,
        }
    }

    pub fn active(&self) -> bool {
        self.worker.is_some()
    }

    pub fn start(&mut self) -> Option<Sender<Input>> {
        if self.active() {
            return None;
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
            Ok(mut target) => {
                self.slot = target.get_user_index().ok();
                let (tx, rx) = mpsc::channel();
                let (stop, stop_rx) = mpsc::channel();
                let (status_tx, status_rx) = mpsc::channel();
                self.stop = Some(stop);
                self.status = Some(status_rx);
                self.worker = Some(thread::spawn(move || {
                    let mut first = true;
                    loop {
                        if stop_rx.try_recv().is_ok() {
                            break;
                        }
                        let timeout = if first {
                            Duration::from_secs(2)
                        } else {
                            Duration::from_secs(1)
                        };
                        match rx.recv_timeout(timeout) {
                            Ok(input) => {
                                first = false;
                                if let Err(error) = target.update(&report(&input)) {
                                    let _ = status_tx
                                        .send(format!("Virtual Xbox update failed: {error:?}"));
                                    break;
                                }
                            }
                            Err(_) => {
                                let _ = status_tx
                                    .send("Xbox output stopped: Sony input timed out.".into());
                                break;
                            }
                        }
                    }
                }));
                return Some(tx);
            }
            Err(vigem_client::Error::BusNotFound) => {
                self.error =
                    Some("ViGEmBus is missing. Preparing the bundled verified installer.".into());
                self.download_driver();
            }
            Err(error) => {
                self.error = Some(format!("Virtual Xbox controller unavailable: {error:?}"))
            }
        }
        None
    }

    pub fn stop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        self.status = None;
        self.slot = None;
    }

    pub fn downloading(&self) -> bool {
        self.download.is_some()
    }

    pub fn poll(&mut self) {
        if let Some(status) = &self.status {
            if let Ok(message) = status.try_recv() {
                self.error = Some(message);
                self.stop();
            }
        }
        let Some(receiver) = &self.download else {
            return;
        };
        if let Ok(result) = receiver.try_recv() {
            self.download = None;
            match result {
                Ok(path) => {
                    self.error = Some("Bundled ViGEmBus installer is ready. Open it to install the driver, then retry Xbox output.".into());
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
    if !verified(DRIVER) {
        return Err("Bundled installer SHA-256 did not match the pinned official release".into());
    }
    let temporary = directory.join(format!("{DRIVER_NAME}.partial"));
    fs::write(&temporary, DRIVER).map_err(|e| e.to_string())?;
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
