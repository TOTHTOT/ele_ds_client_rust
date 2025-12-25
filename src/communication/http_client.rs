use crate::communication::http_client::communication::{
    GeneralHttpRequest, GeneralHttpResponse, RequestUserInfo,
};
use embedded_svc::http::client::Response;
use embedded_svc::{http::client::Client, io::Write, utils::io};
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use std::ops::Add;

// 和服务器通信的基本数据包
pub mod communication {
    use serde::{Deserialize, Serialize};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct GeneralHttpResponse {
        pub seq: u64,
        pub cmd: String,
        pub timestamp: u64,
        pub payload: serde_json::Value,
    }

    impl GeneralHttpResponse {
        pub fn get_now_timestamp() -> u64 {
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(time) => time.as_secs(),
                Err(_) => 0,
            }
        }

        pub fn new_response(seq: u64, cmd: String, payload: serde_json::Value) -> Self {
            Self {
                seq,
                cmd,
                timestamp: Self::get_now_timestamp(),
                payload,
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct RequestUserInfo {
        pub username: String,
        pub password: String,
    }
    impl Default for RequestUserInfo {
        fn default() -> Self {
            Self {
                username: "test".to_string(),
                password: "test".to_string(),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct GeneralHttpRequest {
        pub user_info: RequestUserInfo,
        pub timestamp: u64,
        pub seq: u64,
        pub payload: serde_json::Value,
    }
}

pub struct EleDsHttpClient {
    client: Client<EspHttpConnection>,
    seq: u64,
    pub server_address: String,
}
impl EleDsHttpClient {
    pub fn new(server_address: &str) -> anyhow::Result<Self> {
        let is_https = server_address.to_lowercase().starts_with("https");

        let config = Configuration {
            use_global_ca_store: false,
            crt_bundle_attach: None,

            // 如果是 HTTPS，缓冲区必须调大，建议收 10KB，发 4KB
            buffer_size: if is_https { Some(10240) } else { Some(4096) },
            buffer_size_tx: if is_https { Some(4096) } else { Some(2048) },

            timeout: Some(std::time::Duration::from_secs(30)),
            ..Default::default()
        };

        let connection = EspHttpConnection::new(&config)
            .map_err(|e| anyhow::anyhow!("Connection init failed: {:?}", e))?;

        let client = Client::wrap(connection);

        Ok(Self {
            client,
            seq: 0,
            server_address: server_address.to_string(),
        })
    }

    /// 发送一个post请求数据格式是json
    pub fn post_msg(
        &mut self,
        path: &str,
        msg: serde_json::Value,
    ) -> anyhow::Result<(u16, String)> {
        let request_str = serde_json::to_string(&GeneralHttpRequest {
            user_info: RequestUserInfo::default(),
            timestamp: GeneralHttpResponse::get_now_timestamp(),
            seq: self.seq.add(1),
            payload: msg,
        })?;

        let headers = [("content-type", "application/json")];
        let url = format!("{}{}", self.server_address, path);
        log::info!("posting to {}", url);
        let mut request = self.client.post(url.as_str(), &headers)?;
        request.write_all(request_str.as_bytes())?;
        request.flush()?;
        let mut response = request.submit()?;

        let status = response.status();
        log::info!("status: {}", status);

        // 读取任意长度数据并保存到 recv_vec
        let mut recv_vec = Vec::new();
        loop {
            let mut buf = [0u8; 256];
            match io::try_read_full(&mut response, &mut buf) {
                Ok(0) => break, // 读到0说明读完数据了
                Ok(n) => {
                    recv_vec.extend_from_slice(&buf[..n]);
                    n
                }
                Err(e) => return Err(anyhow::anyhow!("post_msg read response failed, {}", e.0)),
            };
        }
        let body_string = String::from_utf8(recv_vec)?;
        Ok((status, body_string.to_string()))
    }

    pub fn get_file<F>(&mut self, path: &str, mut handle: F) -> anyhow::Result<(), anyhow::Error>
    where
        F: FnMut(Response<&mut EspHttpConnection>) -> anyhow::Result<()>,
    {
        let url = format!("{}/{}", self.server_address, path);
        log::info!("Start download file from: {}", url);

        let request = self.client.get(url.as_str())?;

        let response = request.submit()?;

        let status = response.status();
        if status != 200 {
            anyhow::bail!("get file failed: {status}");
        }
        log::info!("status: {}", status);
        handle(response)?;
        log::info!("handle response success");
        Ok(())
    }
}
