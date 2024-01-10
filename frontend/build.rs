use std::process::Command;
use std::path::Path;
use std::fs;

fn main() {
    let frontend_path = Path::new("../frontend");
    let output_path = Path::new("../target/wasm");
    let js_path = Path::new("../public/js");
    
    let status = Command::new("wasm-pack")
        .arg("build")
        .arg("--target")
        .arg("web")
        .arg("--out-dir")
        .arg(output_path)
        .current_dir(&frontend_path)
        .status()
        .expect("Failed to run wasm-pack build");

    assert!(status.success());

    fs::copy(output_path.join("file_link_frontend_bg.wasm"), js_path.join("file_link_frontend_bg.wasm"))
        .expect("Failed to copy WASM file to /public/js");

    fs::copy(output_path.join("file_link_frontend.js"), js_path.join("file_link.js"))
        .expect("Failed to copy WASM JS file to /public/js");

    let rollup_command = format!("rollup  {}/main.js --format iife --file {}/bundle.js", frontend_path.to_string_lossy(), js_path.to_string_lossy());
    let output = Command::new("cmd")
        .args(&["/C", &rollup_command])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("Rollup command failed with output:\n{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
} 