use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    build_library();
}

fn build_library() {
    let manifest_path = env::var("CARGO_MANIFEST_DIR").expect("failed to get manifest directory");
    let out_path = env::var("OUT_DIR").expect("missing OUT_DIR");

    // Get the library folder
    let manifest_path = Path::new(&manifest_path);
    let out_path = Path::new(&out_path);

    // Path to the bridge and bridge output
    let project_path = manifest_path.join("lhm-bridge");
    let intermediate_path = out_path.join("lhm-bridge-build");
    let publish_path = out_path.join("lhm-bridge");

    // Path to the nuget packages
    let nuget_path = out_path.join(".nuget");

    // Change nuget packages path
    unsafe {
        env::set_var("NUGET_PACKAGES", nuget_path.clone());
    }

    // Run the build command
    let status = Command::new("dotnet")
        .arg("publish")
        .arg("-c")
        .arg("Release")
        .arg("-r")
        .arg("win-x64")
        .arg("--self-contained")
        .arg("/t:LinkNative")
        .arg("/p:PublishAot=true")
        .arg("/p:NativeLib=Static")
        .arg(format!("/p:OutputPath={}/", publish_path.display()))
        .arg(format!(
            "/p:BaseIntermediateOutputPath={}/",
            intermediate_path.display()
        ))
        .arg("--output")
        .arg(&publish_path)
        .arg(&project_path)
        .status()
        .expect("Failed to execute dotnet publish");

    // Build the C# project
    if !status.success() {
        panic!("failed to build binding library");
    }

    // Get the AOT SDK path
    let sdk_path = nuget_path
        .join("runtime.win-x64.microsoft.dotnet.ilcompiler")
        .join("7.0.0")
        .join("sdk");

    // Re-run build script if the bridge code changes
    println!(
        "cargo:rerun-if-changed={}",
        project_path.join("Bridge.cs").display()
    );

    // Setup linker paths
    println!("cargo:rustc-link-search=native={}", publish_path.display());
    println!("cargo:rustc-link-search=native={}", sdk_path.display());

    // Link to required C# AOT libraries
    println!("cargo:rustc-link-lib=static=bootstrapperdll");
    println!("cargo:rustc-link-lib=static=Runtime.WorkstationGC");
    println!("cargo:rustc-link-lib=static=System.Globalization.Native.Aot");
    println!("cargo:rustc-link-lib=static=System.IO.Compression.Native.Aot");

    // Link to bridge library
    println!("cargo:rustc-link-lib=static=lhm-bridge");

    // Link to required system libraries
    println!("cargo:rustc-link-lib=dylib=Iphlpapi");
    println!("cargo:rustc-link-lib=dylib=Crypt32");
    println!("cargo:rustc-link-lib=dylib=advapi32");
    println!("cargo:rustc-link-lib=dylib=bcrypt");
    println!("cargo:rustc-link-lib=dylib=ole32");
    println!("cargo:rustc-link-lib=dylib=oleaut32");
}
