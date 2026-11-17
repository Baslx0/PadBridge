use crate::sony::Input;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    net::{SocketAddr, TcpListener},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};
use viiper_client::{devices::xbox360::Xbox360Input, types::DeviceCreateRequest, ViiperClient};

const SERVER: &[u8] = include_bytes!("../third_party/VIIPER/viiper-windows-amd64.exe");
const SERVER_HASH: &str = "142c144f6abee8b8a5f6a1abbc7018683444de71d2e45102cde6aaab9b2223e6";
const DRIVER: &[u8] = include_bytes!("../third_party/usbip-win2/USBip-0.9.8.1-x64.exe");
const DRIVER_HASH: &str = "38cad6d4432b52d5bb9409d9ad03b72fdffc4ada4cd3a48fbeca1a2752a8518a";

pub struct XboxOutput {
    worker: Option<JoinHandle<()>>,
    stop: Option<Sender<()>>,
    status: Option<Receiver<OutputStatus>>,
    ready: bool,
    prepare: Option<Receiver<Result<PathBuf, String>>>,
    pub error: Option<String>,
    pub installer: Option<PathBuf>,
    pub slot: Option<u32>,
}

enum OutputStatus {
    Ready(u32),
    Failed(String),
}

impl XboxOutput {
    pub fn new() -> Self {
        Self {
            worker: None,
            stop: None,
            status: None,
            ready: false,
            prepare: None,
            error: None,
            installer: None,
            slot: None,
        }
    }
    pub fn active(&self) -> bool {
        self.ready
    }
    pub fn busy(&self) -> bool {
        self.worker.is_some()
    }
    pub fn downloading(&self) -> bool {
        self.prepare.is_some()
    }

    pub fn start(&mut self) -> Option<Sender<Input>> {
        if self.busy() {
            return None;
        }
        self.error = None;
        let Some(usbip) = usbip_path() else {
            self.error = Some("USB/IP driver is missing. Preparing the bundled installer.".into());
            self.prepare_installer();
            return None;
        };
        self.installer = None;
        let (tx, rx) = mpsc::channel();
        let (stop, stop_rx) = mpsc::channel();
        let (status_tx, status_rx) = mpsc::channel();
        self.stop = Some(stop);
        self.status = Some(status_rx);
        self.worker = Some(thread::spawn(move || match launch(&usbip) {
            Ok((mut server, mut stream, slot)) => {
                if stop_rx.try_recv().is_ok() {
                    let _ = server.kill();
                    let _ = server.wait();
                    return;
                }
                let _ = status_tx.send(OutputStatus::Ready(slot));
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
                            if let Err(error) = stream.send(&report(&input)) {
                                let _ = status_tx.send(OutputStatus::Failed(format!(
                                    "VIIPER input stream failed: {error}"
                                )));
                                break;
                            }
                        }
                        Err(_) => {
                            let _ = status_tx.send(OutputStatus::Failed(
                                "Xbox output stopped: Sony input timed out.".into(),
                            ));
                            break;
                        }
                    }
                }
                drop(stream);
                let _ = server.kill();
                let _ = server.wait();
            }
            Err(error) => {
                let _ = status_tx.send(OutputStatus::Failed(error));
            }
        }));
        Some(tx)
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
        self.ready = false;
    }

    pub fn poll(&mut self) {
        if let Some(status) = &self.status {
            if let Ok(event) = status.try_recv() {
                match event {
                    OutputStatus::Ready(slot) => {
                        self.slot = Some(slot);
                        self.ready = true;
                    }
                    OutputStatus::Failed(message) => {
                        self.error = Some(message);
                        self.stop();
                    }
                }
            }
        }
        if let Some(prepare) = &self.prepare {
            if let Ok(result) = prepare.try_recv() {
                self.prepare = None;
                match result {
                    Ok(path) => {
                        self.installer = Some(path);
                        self.error = Some("Bundled USB/IP driver installer is ready. Install it, then retry Xbox output.".into());
                    }
                    Err(error) => {
                        self.error = Some(format!("Could not prepare USB/IP installer: {error}"))
                    }
                }
            }
        }
    }

    fn prepare_installer(&mut self) {
        if self.downloading() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.prepare = Some(rx);
        thread::spawn(move || {
            let _ = tx.send(prepare_driver());
        });
    }
}

fn runtime_dir() -> Result<PathBuf, String> {
    Ok(
        PathBuf::from(env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is unavailable")?)
            .join("PadBridge")
            .join("Runtime"),
    )
}

fn checked_file(name: &str, bytes: &[u8], hash: &str) -> Result<PathBuf, String> {
    if format!("{:x}", Sha256::digest(bytes)) != hash {
        return Err(format!("Bundled {name} failed SHA-256 verification"));
    }
    let dir = runtime_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(name);
    if fs::read(&path)
        .ok()
        .is_some_and(|existing| existing == bytes)
    {
        return Ok(path);
    }
    fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path)
}

fn prepare_driver() -> Result<PathBuf, String> {
    let path = checked_file("USBip-0.9.8.1-x64.exe", DRIVER, DRIVER_HASH)?;
    fs::write(
        runtime_dir()?.join("usbip-win2-LICENSE.txt"),
        include_bytes!("../third_party/usbip-win2/LICENSE.txt"),
    )
    .map_err(|e| e.to_string())?;
    Ok(path)
}

fn prepare_server() -> Result<PathBuf, String> {
    let path = checked_file("viiper-windows-amd64.exe", SERVER, SERVER_HASH)?;
    let dir = runtime_dir()?;
    fs::write(
        dir.join("VIIPER-LICENSE.txt"),
        include_bytes!("../third_party/VIIPER/LICENSE.txt"),
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        dir.join("VIIPER-source-v0.9.0.zip"),
        include_bytes!("../third_party/VIIPER/source-v0.9.0.zip"),
    )
    .map_err(|e| e.to_string())?;
    Ok(path)
}

fn usbip_path() -> Option<PathBuf> {
    if let Some(program_files) = env::var_os("ProgramFiles") {
        let path = PathBuf::from(program_files).join("USBip").join("usbip.exe");
        if path.is_file() {
            return Some(path);
        }
    }
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|dir| dir.join("usbip.exe"))
            .find(|p| p.is_file())
    })
}

fn launch(usbip: &Path) -> Result<(Child, viiper_client::DeviceStream, u32), String> {
    let occupied = [0, 1, 2, 3].map(xinput_connected);
    let server_path = prepare_server()?;
    let usb_listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let api_listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let usb_port = usb_listener.local_addr().map_err(|e| e.to_string())?.port();
    let api_port = api_listener.local_addr().map_err(|e| e.to_string())?.port();
    drop(usb_listener);
    drop(api_listener);
    let old_path = env::var_os("PATH").unwrap_or_default();
    let paths = env::join_paths(
        std::iter::once(
            usbip
                .parent()
                .ok_or("USB/IP path is invalid")?
                .to_path_buf(),
        )
        .chain(env::split_paths(&old_path)),
    )
    .map_err(|e| e.to_string())?;
    let mut command = Command::new(server_path);
    let log_path = runtime_dir()?.join(format!("viiper-{}.log", std::process::id()));
    command
        .arg("server")
        .arg(format!("--usb.addr=127.0.0.1:{usb_port}"))
        .arg(format!("--api.addr=127.0.0.1:{api_port}"))
        .arg(format!("--log.file={}", log_path.display()))
        .arg("--no-tray")
        .env("PATH", paths)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let client = ViiperClient::new(SocketAddr::from(([127, 0, 0, 1], api_port)));
    let mut ready = false;
    for _ in 0..30 {
        if client.ping().is_ok() {
            ready = true;
            break;
        }
        if child.try_wait().map_err(|e| e.to_string())?.is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    if !ready {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!(
            "VIIPER server did not start. Check {}",
            log_path.display()
        ));
    }
    let result = (|| {
        let bus = client.bus_create(None).map_err(|e| e.to_string())?.bus_id;
        let request = DeviceCreateRequest {
            r#type: Some("xbox360".into()),
            id_vendor: None,
            id_product: None,
            device_specific: None,
        };
        let device = client
            .bus_device_add(bus, &request)
            .map_err(|e| e.to_string())?;
        let mut stream = client
            .connect_device(bus, &device.dev_id)
            .map_err(|e| e.to_string())?;
        stream
            .send(&Xbox360Input::default())
            .map_err(|e| e.to_string())?;
        let mut slot = None;
        for _ in 0..50 {
            slot = (0..4).find(|index| !occupied[*index as usize] && xinput_connected(*index));
            if slot.is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
        let slot = slot.ok_or_else(|| {
            if occupied.contains(&true) {
                format!("USB/IP attached the controller, but no new XInput slot appeared. Another Xbox controller or PadBridge instance is active; close it and retry. Log: {}", log_path.display())
            } else {
                format!("USB/IP attached the controller, but Windows XInput did not detect it within five seconds. Log: {}", log_path.display())
            }
        })?;
        Ok::<_, String>((stream, slot))
    })();
    match result {
        Ok((stream, slot)) => Ok((child, stream, slot)),
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            Err(error)
        }
    }
}

#[link(name = "xinput")]
extern "system" {
    fn XInputGetState(index: u32, state: *mut u8) -> u32;
}
fn xinput_connected(index: u32) -> bool {
    let mut state = [0u8; 16];
    unsafe { XInputGetState(index, state.as_mut_ptr()) == 0 }
}

fn stick(value: f32) -> i16 {
    (value.clamp(-1.0, 1.0) * if value < 0.0 { 32768.0 } else { 32767.0 }) as i16
}
fn report(input: &Input) -> Xbox360Input {
    let mut buttons = 0u32;
    for (index, flag) in [
        (0, 0x8000),
        (1, 0x4000),
        (2, 0x1000),
        (3, 0x2000),
        (4, 0x0100),
        (5, 0x0200),
        (6, 0x0020),
        (7, 0x0010),
        (8, 0x0001),
        (9, 0x0002),
        (10, 0x0004),
        (11, 0x0008),
        (12, 0x0400),
    ] {
        if input.buttons[index] {
            buttons |= flag;
        }
    }
    Xbox360Input {
        buttons,
        lt: (input.triggers[0].clamp(0.0, 1.0) * 255.0) as u8,
        rt: (input.triggers[1].clamp(0.0, 1.0) * 255.0) as u8,
        lx: stick(input.axes[0]),
        ly: stick(input.axes[1]),
        rx: stick(input.axes[2]),
        ry: stick(input.axes[3]),
        reserved: [0; 6],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_installers_match_pinned_hashes() {
        assert_eq!(format!("{:x}", Sha256::digest(DRIVER)), DRIVER_HASH);
        assert_eq!(format!("{:x}", Sha256::digest(SERVER)), SERVER_HASH);
    }

    #[test]
    #[ignore = "requires installed usbip-win2 driver and creates a temporary virtual controller"]
    fn viiper_exposes_new_xinput_slot() {
        let usbip = usbip_path().expect("usbip-win2 must be installed");
        let (mut server, stream, slot) =
            launch(&usbip).expect("VIIPER should attach an Xbox controller");
        assert!(xinput_connected(slot));
        drop(stream);
        server.kill().expect("stop VIIPER");
        server.wait().expect("reap VIIPER");
    }

    #[test]
    fn maps_sony_controls_to_xbox_report() {
        let mut input = Input {
            axes: [-1.0, 1.0, 0.5, -0.5],
            triggers: [1.0, 0.25],
            buttons: [false; 14],
        };
        input.buttons[2] = true;
        input.buttons[4] = true;
        input.buttons[8] = true;
        input.buttons[13] = true;
        let xbox = report(&input);
        assert_eq!(xbox.buttons, 0x1101);
        assert_eq!((xbox.lt, xbox.rt), (255, 63));
        assert_eq!(
            (xbox.lx, xbox.ly, xbox.rx, xbox.ry),
            (i16::MIN, i16::MAX, 16383, -16384)
        );
    }
}
