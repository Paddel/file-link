use std::process::Command;
use std::path::Path;

fn main() {
    let frontend_path = Path::new("../frontend");
    
    let status = Command::new("wasm-pack")
        .arg("build")
        .arg("--target")
        .arg("web")
        .arg("--out-dir")
        .arg("../public/wasm")
        .current_dir(&frontend_path)
        .status()
        .expect("Failed to run wasm-pack build");

    assert!(status.success());
    
    let rollup_command = "rollup ../frontend/main.js --format iife --file ../public/js/bundle.js";
    let output = Command::new("cmd")
        .args(&["/C", &rollup_command])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("Rollup command failed with output:\n{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
}
