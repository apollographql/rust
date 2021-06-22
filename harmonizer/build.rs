use std::fs::metadata;
use std::process::Command;

fn main() {
    if metadata("dist/bridge.js").is_err() {
        assert!(Command::new("npm")
            .current_dir("../")
            .args(&["install"])
            .status()
            .unwrap()
            .success());
        assert!(Command::new("npm")
            .current_dir("../")
            .args(&["run", "compile:for-harmonizer-build-rs"])
            .status()
            .unwrap()
            .success());
    }
}
