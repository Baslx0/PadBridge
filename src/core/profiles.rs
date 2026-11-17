use super::mapping::Mapping;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};
#[derive(Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub mapping: Mapping,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct DeviceProfiles {
    pub active: usize,
    pub profiles: Vec<Profile>,
}
impl Default for DeviceProfiles {
    fn default() -> Self {
        Self {
            active: 0,
            profiles: vec![Profile {
                name: "Default".into(),
                mapping: Mapping::default(),
            }],
        }
    }
}
impl DeviceProfiles {
    pub fn active(&self) -> &Profile {
        &self.profiles[self.active]
    }
    pub fn active_mut(&mut self) -> &mut Profile {
        &mut self.profiles[self.active]
    }
    pub fn valid(&self) -> bool {
        !self.profiles.is_empty()
            && self.active < self.profiles.len()
            && self
                .profiles
                .iter()
                .all(|p| !p.name.trim().is_empty() && p.name.len() <= 80 && p.mapping.valid())
    }
}
#[derive(Serialize, Deserialize)]
pub struct Store {
    version: u32,
    pub devices: BTreeMap<String, DeviceProfiles>,
}
impl Default for Store {
    fn default() -> Self {
        Self {
            version: 1,
            devices: BTreeMap::new(),
        }
    }
}
impl Store {
    pub fn device(&mut self, key: &str) -> &mut DeviceProfiles {
        self.devices.entry(key.into()).or_default()
    }
    pub fn load(path: &Path) -> (Self, Option<String>) {
        match fs::read_to_string(path) { Ok(s) => match serde_json::from_str::<Self>(&s) { Ok(store) if store.version == 1 && store.devices.values().all(DeviceProfiles::valid) => (store,None), _ => (Self::default(),Some("Invalid or outdated profile file; defaults are in use. Original file was preserved.".into())) }, Err(e) if e.kind()==io::ErrorKind::NotFound => (Self::default(),None), Err(e) => (Self::default(),Some(format!("Profile read failed: {e}"))) }
    }
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_vec_pretty(self)?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, data)?;
        fs::rename(tmp, path)
    }
    pub fn path() -> PathBuf {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("PadBridge")
            .join("profiles.json")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_active_index() {
        let d = DeviceProfiles {
            active: 9,
            ..DeviceProfiles::default()
        };
        assert!(!d.valid());
    }
    #[test]
    fn roundtrip() {
        let mut s = Store::default();
        s.device("test").active_mut().mapping.deadzone = 0.2;
        let s: Store = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(s.devices["test"].active().mapping.deadzone, 0.2);
    }
}
