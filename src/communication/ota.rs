use crate::communication::http_client::{communication, EleDsHttpClient};
use crate::device_config::DeviceInfo;
use chrono::NaiveDateTime;
use esp_idf_svc::io;
use esp_idf_svc::ota::EspOta;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

const REQUERY_WHETHER_UPGRADE: &str = "/upgrade/query";
pub struct Ota {
    http_client: Arc<Mutex<EleDsHttpClient>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpgradeQueryResponse {
    pub version: String, // 服务器最新版本
    pub device_type: String,
    pub pack_size: u64,
    pub download_url: String,
}

impl Ota {
    pub fn new(http_client: Arc<Mutex<EleDsHttpClient>>) -> anyhow::Result<Self> {
        Ok(Ota { http_client })
    }
    pub fn is_need_upgrade(&self) -> anyhow::Result<Option<UpgradeQueryResponse>> {
        let device_info = serde_json::json!(&DeviceInfo::default());
        let mut client = self
            .http_client
            .lock()
            .map_err(|e| anyhow::anyhow!("is_need_upgrade() lock client fail, {e}"))?;
        let (statue, response_json) = client.post_msg(REQUERY_WHETHER_UPGRADE, device_info)?;
        if statue != 200 {
            return Err(anyhow::anyhow!(
                "get upgrade response failed, {response_json}"
            ));
        }
        // 拿到服务器返回的数据, 之后根据cmd类型解包
        let response: communication::GeneralHttpResponse = serde_json::from_str(&response_json)?;

        if response.cmd == "UpgradeQuery_ACK" {
            let upgrade_query: UpgradeQueryResponse =
                serde_json::from_str(&response.payload.to_string())?;
            if Ota::judge_version(upgrade_query.version.as_str()) {
                Ok(Some(upgrade_query))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    pub fn get_upgrade_file(&self, upgrade: &UpgradeQueryResponse) -> anyhow::Result<()> {
        let mut client = self
            .http_client
            .lock()
            .map_err(|e| anyhow::anyhow!("get_upgrade_file() lock failed: {e}"))?;

        log::info!(
            "server addr: {}, path: {}",
            client.server_address,
            upgrade.download_url,
        );

        // 开始下载文件
        client.get_file(
            upgrade.download_url.as_str(),
            move |response| -> anyhow::Result<()> {
                let mut buffer = [0_u8; 1024];
                let mut ota = EspOta::new()
                    .map_err(|e| anyhow::anyhow!("Failed to create ota client: {e}"))?;
                let mut update = ota
                    .initiate_update()
                    .map_err(|e| anyhow::anyhow!("failed to initiate update: {e}"))?;
                match io::utils::copy(response, &mut update, &mut buffer) {
                    Ok(_) => {
                        // 下载完成后就重启
                        update.complete()?;
                        ota.mark_running_slot_valid()?;
                        log::info!("Successfully updated");
                        esp_idf_svc::hal::reset::restart();
                        // Ok(())
                    }
                    Err(e) => {
                        anyhow::bail!("failed to copy response: {e}");
                    }
                }
            },
        )?;
        Ok(())
    }

    /// 将时间转为时间戳比较大小, 如果需要更新就返回true
    fn judge_version(remote_time: &str) -> bool {
        let version_str = remote_time.replace(".bin", "");
        let format = "%Y-%m-%d %H:%M:%S";

        // BUILD_TIME 是编译时常量，格式已知正确
        let build_time = match NaiveDateTime::parse_from_str(env!("BUILD_TIME"), format) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to parse BUILD_TIME: {e}, this is a build configuration error");
                return false;
            }
        };

        // remote_time 来自服务器，解析失败时不升级
        let remote_parsed = match NaiveDateTime::parse_from_str(&version_str, format) {
            Ok(t) => t,
            Err(e) => {
                log::warn!("Failed to parse remote version time '{version_str}': {e}");
                return false;
            }
        };
        log::info!("build_time: {build_time}, remote_time: {remote_parsed}");
        remote_parsed > build_time
    }

    /// 同步服务器版本
    pub fn sync_firmware(&self) -> anyhow::Result<()> {
        let is_need_upgrade = self.is_need_upgrade()?;
        let Some(t) = is_need_upgrade else {
            log::info!("System is last");
            return Ok(());
        };
        log::info!("System need upgrade, {t:?}");
        self.get_upgrade_file(&t)
    }
}
