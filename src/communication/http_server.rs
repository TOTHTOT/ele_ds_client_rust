use std::fs;
use std::path::PathBuf;

pub struct HttpServer {}

impl HttpServer {
    pub fn new() -> anyhow::Result<HttpServer> {
        let _ = HttpServer::get_dir_file_path("/fat/");
        Ok(Self {})
    }

    fn get_dir_file_path(path: &str) -> anyhow::Result<Vec<PathBuf>> {
        let path_vec = Vec::<PathBuf>::new();
        match fs::read_dir(path) {
            Ok(entries) => {
                log::info!("{path}: {:?}", entries)
            }
            Err(e) => {
                log::warn!("get_dir_file_path(): {e}")
            }
        }
        Ok(path_vec)
    }
}
