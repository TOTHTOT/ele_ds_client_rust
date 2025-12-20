use embedded_svc::http::client::Response;
use embedded_svc::{http::client::Client, io::Write, utils::io};
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

    /// 发送一个post请求数据格式是json
    pub fn post_msg(&mut self, path: &str, msg: &str) -> anyhow::Result<(u16, String)> {
        let headers = [
            ("content-type", "application/json"),
            ("content-length", &*msg.len().to_string()),
        ];
        let url = format!("{}{}", self.server_address, path);
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

    pub fn get_file<F>(&mut self, path: &str, mut handle: F) -> anyhow::Result<()>
    where
        F: FnMut(Response<&mut EspHttpConnection>),
    {
        let url = format!("{}{}", self.server_address, path);
        log::info!("正在从 URL 下载文件: {}", url);

        // 1. 发起 GET 请求
        // 注意：GET 请求通常不需要设置 content-length，headers 留空即可
        let request = self.client.get(url.as_str())?;

        // 2. 提交请求并获取响应
        let response = request.submit()?;

        // 3. 检查状态码
        let status = response.status();
        if status != 200 {
            anyhow::bail!("get file failed: {status}");
        }

        // 4. 返回连接对象本身，因为它实现了 io::Read 特性
        // 注意：在 esp-idf-svc 中，submit() 返回的是包装后的 Connection
        // 我们可以直接返回底层的 reader
        handle(response);
        Ok(())
    }
}
