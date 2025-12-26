use embedded_svc::http::Method;
use embedded_svc::{http::server::Request, io::Write};
use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer};
use std::fs;
use std::fs::FileType;
use std::path::PathBuf;
pub struct HttpServer<'d> {
    server: EspHttpServer<'d>,
}

impl<'d> HttpServer<'d> {
    pub fn new() -> anyhow::Result<HttpServer<'d>> {
        let config = Configuration {
            stack_size: 10240, // 增加栈空间，默认值可能对 Rust 来说太小了
            ..Default::default()
        };

        // 2. 创建服务器实例
        let mut server = EspHttpServer::new(&config)?;
        // server.handler("/fat*", Method::Get, HttpServer::list_directory_handler)?;
        server.handler("/fat*", Method::Get, |req| {
            if let Err(e) = Self::list_directory_handler(req) {
                log::error!("Handler error: {:?}", e);
            }
            Ok(())
        })?;
        Ok(Self { server })
    }

    fn get_dir_file_path(path: &str) -> anyhow::Result<Vec<(PathBuf, FileType)>> {
        let mut path_vec = Vec::<(PathBuf, FileType)>::new();
        match fs::read_dir(path) {
            Ok(entries) => {
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
                log::info!("{path}: {:?}", path_vec)
            }
            Err(e) => {
                log::warn!("get_dir_file_path(): {e}")
            }
        }
        Ok(path_vec)
    }

    fn generate_html(current_path: &str, items: Vec<(PathBuf, std::fs::FileType)>) -> String {
        let mut html = String::new();
        html.push_str(
            "<html><head><meta charset='utf-8'><title>ESP32 File Server</title></head><body>",
        );
        html.push_str(&format!("<h1>当前目录: {}</h1>", current_path));

        // 1. 添加“返回上一级”连接
        if current_path != "/fat/" {
            html.push_str("<p><a href='..'>[ ⬆️ 返回上一级 ]</a></p>");
        }

        html.push_str("<ul>");

        for (path, f_type) in items {
            // 获取文件名（去掉完整的路径前缀）
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                let icon = if f_type.is_dir() { "📁" } else { "📄" };

                // 如果是目录，给路径末尾加上 / 方便浏览器识别路径层级
                let link_name = if f_type.is_dir() {
                    format!("{}/", file_name)
                } else {
                    file_name.to_string()
                };

                // 生成超链接：<a href="文件名">图标 文件名</a>
                html.push_str(&format!(
                    "<li>{} <a href='{}'>{}</a></li>",
                    icon, link_name, link_name
                ));
            }
        }

        html.push_str("</ul></body></html>");
        html
    }

    // 处理文件列表请求的回调函数
    pub fn list_directory_handler(req: Request<&mut EspHttpConnection>) -> anyhow::Result<()> {
        // 1. 获取当前请求的路径，如果没有则默认为 /fat/
        let mut uri = req.uri().to_string();
        if uri.is_empty() {
            uri = "/fat/".to_string();
        }

        // 确保路径以 / 结尾，这对浏览器的 ".." 相对路径逻辑至关重要
        if !uri.ends_with('/') {
            uri.push('/');
        }

        log::info!("Handling request for path: {}", uri);

        // 2. 获取目录下的文件列表
        let path_vec = Self::get_dir_file_path(&uri).unwrap_or_default();

        // 3. 开始发送 HTTP 响应
        let mut response = req.into_ok_response()?;

        // 为了节省内存，我们分段写入 response，而不是构造一个巨大的 String
        response.write_all(
            b"<html><head><meta charset='utf-8'><style>\
            body { font-family: sans-serif; line-height: 1.6; padding: 20px; }\
            a { text-decoration: none; color: #007bff; }\
            li { list-style: none; margin-bottom: 8px; }\
            </style></head><body>",
        )?;

        response.write_all(format!("<h1>目录索引: {}</h1>", uri).as_bytes())?;

        // 4. 添加“返回上一级”
        if uri != "/fat/" {
            response.write_all(b"<div><a href='..'>[ \xE2\xAC\x85 return ]</a></div><hr>")?;
        }

        response.write_all(b"<ul>")?;

        // 5. 遍历并发送列表项
        for (path, f_type) in path_vec {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let icon = if f_type.is_dir() { "dir" } else { "file" };

                // 目录链接需要带 /
                let link_path = if f_type.is_dir() {
                    format!("{}/", name)
                } else {
                    name.to_string()
                };

                let line = format!(
                    "<li>{} <a href='{}'>{}</a></li>",
                    icon, link_path, link_path
                );
                response.write_all(line.as_bytes())?;
            }
        }

        response.write_all(b"</ul></body></html>")?;

        Ok(())
    }
}
