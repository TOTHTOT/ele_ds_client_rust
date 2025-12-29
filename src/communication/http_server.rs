use embedded_svc::http::Method;
use embedded_svc::{http::server::Request, io::Write};
use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer};
use std::fs;
use std::fs::FileType;
use std::path::PathBuf;

#[allow(dead_code)]
pub struct HttpServer<'d> {
    server: EspHttpServer<'d>,
}
#[allow(dead_code)]
impl<'d> HttpServer<'d> {
    pub fn new() -> anyhow::Result<HttpServer<'d>> {
        let config = Configuration {
            stack_size: 10240,
            uri_match_wildcard: true,
            ..Default::default()
        };
        let mut server = EspHttpServer::new(&config)?;
        server.fn_handler("/fat*", Method::Get, |req| {
            Self::list_directory_handler(req)
        })?;
        Ok(Self { server })
    }

    /// 根据传入路径获取路径内文件夹和文件并回传
    fn get_dir_contents_with_path(path: &str) -> anyhow::Result<Vec<(PathBuf, FileType)>> {
        let mut path_vec = Vec::<(PathBuf, FileType)>::new();
        let path_buf = PathBuf::from(&path);
        let metadata = fs::metadata(&path_buf)?;

        // 如果不是文件夹就直接返回文件类型
        if !metadata.is_dir() {
            let actual_type = metadata.file_type();
            anyhow::bail!("{:?}", actual_type);
        }

        let entries = fs::read_dir(&path_buf)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if let Ok(metadata) = fs::metadata(&path) {
                let f_type = metadata.file_type();
                path_vec.push((path, f_type));
            } else {
                log::warn!("Could not get metadata for {:?}", path);
            }
        }
        log::info!("{path}: {:?}", path_vec);
        Ok(path_vec)
    }

    /// 生成文件和目录的 html 字符串
    fn generate_dir_file_html(current_path: &str, items: &Vec<(PathBuf, FileType)>) -> String {
        let mut html = String::new();
        html.push_str(
            "<html><head><meta charset='utf-8'><title>ESP32 File Server</title></head><body>",
        );
        html.push_str(&format!("<h1>current direct: {}</h1>", current_path));

        if current_path != "/fat/" {
            html.push_str("<p><a href='..'>[ ⬆️ return ]</a></p>");
        }
        html.push_str("<ul>");

        for (path, f_type) in items {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                let icon = if f_type.is_dir() { "📁" } else { "📄" };

                let link_name = if path.is_dir() {
                    format!("{}{}/", current_path, file_name)
                } else {
                    format!("{}{}", current_path, file_name)
                };
                log::info!("link_name: {link_name}");

                html.push_str(&format!(
                    "<li>{} <a href='{}'>{}</a></li>",
                    icon, link_name, file_name
                ));
            }
        }

        html.push_str("</ul></body></html>");
        html
    }

    /// 通用失败页面
    fn general_failed_html(failed_str: &str) -> String {
        let mut html = String::new();
        html.push_str("<html><head><meta charset='utf-8'><title>Failed - ESP32</title>");
        html.push_str("<style>body{font-family:sans-serif;padding:20px;line-height:1.6;}\
                   .error-box{border:1px solid #ff4444;background:#fff5f5;padding:15px;border-radius:5px;}\
                   .btn{display:inline-block;padding:8px 15px;background:#007bff;color:white;text-decoration:none;border-radius:4px;}</style>");
        html.push_str("</head><body>");

        html.push_str(failed_str);

        html.push_str("<p style='margin-top:20px;'>");
        html.push_str("<a href='..' class='btn'>[ ⬅️ return ]</a>");
        html.push_str("</p>");

        html.push_str("</body></html>");
        html
    }

    /// 根据路径获取文件夹内容失败时返回
    fn generate_dir_file_failed_html(current_path: &str, error_msg: &str) -> String {
        let mut html = String::new();
        html.push_str("<h1>⚠️ Read path failed</h1>");
        html.push_str("<div class='error-box'>");
        html.push_str(&format!("<p><strong>Path:</strong> {}</p>", current_path));
        html.push_str(&format!("<p><strong>Reason:</strong> {}</p>", error_msg));
        html.push_str("</div>");
        Self::general_failed_html(html.as_str())
    }
    /// 处理文件列表请求的回调函数
    pub fn list_directory_handler(req: Request<&mut EspHttpConnection>) -> anyhow::Result<()> {
        let mut uri = req.uri().to_string();
        // 如果字符串是空的或者只有一个 / 就补全目录, 有的浏览器在没输入路径时自动传入 /
        if uri.is_empty() {
            uri = "/fat/".to_string();
        } else if uri == "/" {
            uri = "/fat/".to_string();
        }
        if !uri.ends_with('/') {
            uri.push('/');
        }

        log::info!("Handling request for path: {}", uri);
        let mut response = req.into_ok_response()?;
        let response_str = match Self::get_dir_contents_with_path(&uri) {
            Ok(path_vec) => Self::generate_dir_file_html(uri.as_str(), &path_vec),
            Err(e) => {
                // 简单判断, 如果打开的是文件就读取文件内容
                if e.to_string().contains("is_file: true") {
                    log::info!("access file");
                    String::new()
                } else {
                    Self::generate_dir_file_failed_html(uri.as_str(), format!("{}", e).as_str())
                }
            }
        };
        response.write_all(response_str.as_bytes())?;
        Ok(())
    }
}
