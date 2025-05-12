use std::{env, fs, path::Path, process::Command};

fn main() {
    build_library();
}

fn build_library() {
    let manifest_path = env::var("CARGO_MANIFEST_DIR").expect("failed to get manifest directory");
    let out_path = env::var("OUT_DIR").unwrap();

    // Get the library folder
    let manifest_path = Path::new(&manifest_path);
    let out_path = Path::new(&out_path);

    // Path to the bridge and bridge output
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

    // Get the built DLL
    let output_file = publish_path.join("lhm-bridge.dll");

    // Check the build output dll exists
    if !output_file.exists() {
        panic!("build output not found");
    }

    // Copy dll to output directory
    fs::copy(output_file, out_path.join("lhm-bridge.dll"))
        .expect("Failed to copy .dll to Rust target directory");

    // Re-run build script if the bridge code changes
    println!(
        "cargo:rerun-if-changed={}",
        project_path.join("Bridge.cs").display()
    );
}
