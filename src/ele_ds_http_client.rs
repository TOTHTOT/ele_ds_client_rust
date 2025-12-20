use embedded_svc::http::client::Response;
use embedded_svc::{http::client::Client, io::Write, utils::io};
use esp_idf_svc::http::client::EspHttpConnection;
pub struct EleDsHttpClient {
    client: Client<EspHttpConnection>,
    pub server_address: String,
}

impl EleDsHttpClient {
    pub fn new(server_address: &str) -> anyhow::Result<Self> {
        let client = Client::wrap(EspHttpConnection::new(&Default::default())?);
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
        let url = format!("{}{}", self.server_address, path);
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
