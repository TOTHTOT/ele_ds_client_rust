use chrono::Datelike;
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
    pub wifi_ssid: Option<String>,
    pub wifi_password: Option<String>,
    pub requery_upgrade_time_minutes: u32, // 查询更新版本间隔, 单位: 分钟
    pub wifi_max_link_time: u8,            // wifi最大连接时间, 秒
    pub time_zone: String,                 // 时区
    pub city_name: String,                 // 所在城市地点, 获取天气
    pub wifi_connect_interval: u32,        // WiFi 连接的电源周期间隔, 和 boot_times 一起用
    pub boot_times: u32,                   // 重启次数
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            user_info: UserInfo::default(),
            device_info: DeviceInfo::default(),
            wifi_ssid: Some("esp-2.4G".to_string()),
            wifi_password: Some("12345678..".to_string()),
            requery_upgrade_time_minutes: 1440,
            wifi_max_link_time: 30,
            time_zone: "CST-8".to_string(),
            city_name: "Fuzhou".to_string(),
            wifi_connect_interval: 60,
            boot_times: 0,
        }
    }
}

impl DeviceConfig {
    /// 加载配置
    pub fn load_config() -> anyhow::Result<DeviceConfig> {
        // Self::delete_config_file()?;
        let config_string = match fs::read_to_string(DEFAULT_DEVICE_CONFIG_FILE_PATH) {
            Ok(string) => string,
            Err(e) => {
                log::info!("load_config failed: {e}");
                return Self::rebuild_device_config();
            }
        };
        let config: DeviceConfig = match serde_json::from_str(&config_string) {
            Ok(config) => config,
            Err(e) => {
                log::warn!("Parse config failed: {e:?}, rebuilding...");
                Self::rebuild_device_config()?
            }
        };
        Ok(config)
    }
    fn rebuild_device_config() -> anyhow::Result<DeviceConfig> {
        if let Some(parent) = std::path::Path::new(DEFAULT_DEVICE_CONFIG_FILE_PATH).parent() {
            fs::create_dir_all(parent)?;
        }

        let default_config = DeviceConfig::default();
        default_config.save_config()?;
        Ok(default_config)
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
        log::info!("config file has beed saved to: {DEFAULT_DEVICE_CONFIG_FILE_PATH}");
        Ok(())
    }

    pub fn delete_config_file() -> anyhow::Result<()> {
        fs::remove_file(DEFAULT_DEVICE_CONFIG_FILE_PATH)?;
        Ok(())
    }
    pub fn set_user_info(&mut self, user_info: UserInfo) {
        self.user_info = user_info;
    }

    pub fn is_need_connect_wifi(&self) -> bool {
        // 没设置间隔时间就每次都连接
        if self.wifi_connect_interval == 0 {
            return true;
        }
        self.boot_times % self.wifi_connect_interval == 0
    }

    /// 启动后增加一次启动次数并保存
    pub fn boot_times_add(&mut self) -> anyhow::Result<()> {
        self.boot_times += 1;
        self.save_config()
    }

    pub fn current_time_is_too_old() -> bool {
        let now = chrono::Local::now();
        now.year() < 2025
    }
}
