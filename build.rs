use chrono::Local;
fn main() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let build_time = std::env::var("BUILD_TIME").unwrap_or_else(|_| now);
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);
    println!("cargo:rerun-if-env-changed=BUILD_TIME");
    embuild::espidf::sysenv::output();
}
