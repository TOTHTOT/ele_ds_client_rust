use embedded_svc::http::client::Response;
use embedded_svc::{http::client::Client, io::Write, utils::io};
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};

// 和服务器通信的基本数据包
pub mod communication {
    use std::time::{SystemTime, UNIX_EPOCH};
    use serde::{Deserialize, Serialize};

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
}

pub struct EleDsHttpClient {
    client: Client<EspHttpConnection>,
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
            server_address: server_address.to_string(),
        })
    }

    /// 发送一个post请求数据格式是json
    pub fn post_msg(&mut self, path: &str, msg: &str) -> anyhow::Result<(u16, String)> {
        let headers = [
            ("content-type", "application/json"),
            ("content-length", &*msg.len().to_string()),
        ];
        let url = format!("{}{}", self.server_address, path);
        log::info!("posting to {}", url);
        let mut request = self.client.post(url.as_str(), &headers)?;
        request.write_all(msg.as_bytes())?;
        request.flush()?;
        let mut response = request.submit()?;

        let status = response.status();
        log::info!("status: {}", status);
        let mut buf = [0u8; 1024];
        let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
        let body_string = std::str::from_utf8(&buf[0..bytes_read])?;
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
