use crate::hardware::mpu::MpuConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const CONFIG_PATH: &str = "lidarino_config.toml";

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Config {
    pub mpu_config: Option<MpuConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            mpu_config: Some(MpuConfig::default()),
        }
    }
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let string = std::fs::read_to_string(path)?;
        *self = toml::from_str(&string)?;
        if self.mpu_config.is_none() {
            self.mpu_config = Some(MpuConfig::default());
        }
        Ok(())
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let string = toml::to_string_pretty(self)?;
        std::fs::write(path, string)?;
        Ok(())
    }
}
