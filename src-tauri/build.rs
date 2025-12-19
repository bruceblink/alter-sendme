use std::{env, fs, path::PathBuf};

fn main() {
    // 尝试获取 workspace 根目录
    let workspace_root = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".")), // 直接返回 PathBuf，避免引用临时值
        Err(_) => {
            eprintln!("Warning: CARGO_MANIFEST_DIR not set, fallback to current directory");
            PathBuf::from(".")
        }
    };

    let mut version = env!("CARGO_PKG_VERSION").to_string();

    // 根目录 package.json
    let package_json_path = workspace_root.join("package.json");
    if let Ok(package_json_str) = fs::read_to_string(&package_json_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&package_json_str) {
            if let Some(v) = json.get("version").and_then(|v| v.as_str()) {
                version = v.to_string();
            }
        }
    } else {
        eprintln!(
            "Warning: package.json not found at {}",
            package_json_path.display()
        );
    }

    // 无条件输出 APP_VERSION
    println!("cargo:rustc-env=APP_VERSION={}", version);
    println!("cargo:rerun-if-changed={}", package_json_path.display());

    // Tauri build
    tauri_build::build();
}
