use embedded_svc::{
    http::client::Client,
    io::Write,
    utils::io,
};
use esp_idf_svc::http::client::EspHttpConnection;
pub struct EleDsHttpClient {
    client: Client<EspHttpConnection>,
    server_address: String,
}

impl EleDsHttpClient {
    pub fn new(server_address: &str) -> anyhow::Result<Self> {
        let client = Client::wrap(EspHttpConnection::new(&Default::default())?);
        Ok(Self {
            client,
            server_address: server_address.to_string(),
        })
    }

    pub fn post_msg(&mut self, msg: &str) -> anyhow::Result<(u16, String)> {
        let headers = [
            ("content-type", "application/json"),
            ("content-length", &*msg.len().to_string()),
        ];

        let mut request = self.client.post(self.server_address.as_str(), &headers)?;
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
}
