use std::process::Command;
use std::fs;
use std::path::Path;

fn main() {
    let frontend_path = Path::new("../frontend");
    let output_path = Path::new("public");

    let frontend_metadata = fs::metadata(&frontend_path).expect("Failed to get frontend metadata");
    let output_metadata = match fs::metadata(&output_path) {
        Ok(meta) => Some(meta),
        Err(_) => None,
    };

    let should_build = match output_metadata {
        Some(om) => frontend_metadata.modified().expect("Failed to get modified time") > om.modified().expect("Failed to get output modified time"),
        None => true,
    };

    if should_build {
        let status = Command::new("wasm-pack")
            .arg("build")
            .arg("--target")
            .arg("web")
            .arg("--out-dir")
            .arg("../public")
            .current_dir(&frontend_path)
            .status()
            .expect("Failed to run wasm-pack build");

        assert!(status.success());
        
        let rollup_command = "rollup ../frontend/main.js --format iife --file ../public/bundle.js";
        let output = Command::new("cmd")
            .args(&["/C", &rollup_command])
            .output()
            .expect("Failed to execute command");

        if !output.status.success() {
            eprintln!("Rollup command failed with output:\n{}", String::from_utf8_lossy(&output.stderr));
            std::process::exit(1);
        }
    }
}
