use std::{env, fs, path::Path, process::Command};

fn main() {
    build_library();
}

fn build_library() {
    // Get the location of the cargo toml
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("failed to get manifest directory");
    let manifest_path = Path::new(&manifest_dir);
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap();
    let out_dir = format!("{}/target/{}", crate_dir, profile);
    let out_path = Path::new(&out_dir);
    let project_path = manifest_path.join("lhm-bridge");
    let publish_path = project_path.join("release");

    // Run the build command
    let status = Command::new("dotnet")
        .arg("publish")
        .arg("-c")
        .arg("Release")
        .arg("-r")
        .arg("win-x64")
        .arg("/p:SelfContained=true")
        .arg("/p:PublishAot=true")
        .arg("--output")
        .arg(&publish_path)
        .arg(&project_path)
        .status()
        .expect("Failed to execute dotnet publish");

    if !status.success() {
        panic!("failed to build binding library");
    }

    println!("built binding library");

    let output_file = publish_path.join("lhm-bridge.dll");

    if output_file.exists() {
        // copy dll to output directory
        fs::copy(output_file, out_path.join("lhm-bridge.dll"))
            .expect("Failed to copy .dll to Rust target directory");
    } else {
        panic!("build output not found");
    }

    // Re-run build script if the bridge code changes
    println!(
        "cargo:rerun-if-changed={}",
        project_path.join("Bridge.cs").display()
    );
}
