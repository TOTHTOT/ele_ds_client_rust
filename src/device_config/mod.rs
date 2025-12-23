use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

pub const DEFAULT_DEVICE_CONFIG_FILE_PATH: &str = "/fat/system/config"; // 默认的配置文件保存地址
#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceInfo {
    version: String,
    device_type: String,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            version: String::from(env!("BUILD_TIME")),
            device_type: String::from("ele_ds_client_rust"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    username: String,
    password: String,
}
impl Default for UserInfo {
    fn default() -> Self {
        Self {
            username: "".to_string(),
            password: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceConfig {
    user_info: UserInfo,
    device_info: DeviceInfo,
    requery_upgrade_time_minutes: u32, // 查询更新版本间隔, 单位: 分钟
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            user_info: UserInfo::default(),
            device_info: DeviceInfo::default(),
            requery_upgrade_time_minutes: 1440,
        }
    }
}

impl DeviceConfig {
    /// 加载配置
    pub fn load_config() -> anyhow::Result<DeviceConfig> {
        let config_string = match fs::read_to_string(DEFAULT_DEVICE_CONFIG_FILE_PATH) {
            Ok(string) => string,
            Err(e) => {
                log::info!("load_config failed: {e}");

                if let Some(parent) = std::path::Path::new(DEFAULT_DEVICE_CONFIG_FILE_PATH).parent() {
                    fs::create_dir_all(parent)?;
                }

                let default_config = DeviceConfig::default();
                default_config.save_config()?;
                return Ok(default_config);
            }
        };
        let config: DeviceConfig = serde_json::from_str(&config_string)?;
        Ok(config)
    }

    /// 保存数据到文件, 这里保存的配置是格式化后的
    pub fn save_config(&self) -> anyhow::Result<()> {
        let config_string = serde_json::to_string_pretty(self)?;
        if let Some(parent) = std::path::Path::new(DEFAULT_DEVICE_CONFIG_FILE_PATH).parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(DEFAULT_DEVICE_CONFIG_FILE_PATH)?;

        file.write_all(config_string.as_bytes())?;
        log::info!("config file has beed saved to: {}", DEFAULT_DEVICE_CONFIG_FILE_PATH);
        Ok(())
    }

    pub fn set_user_info(&mut self, user_info: UserInfo) {
        self.user_info = user_info;
    }
}
