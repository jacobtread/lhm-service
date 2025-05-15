use std::{env, fs, path::Path, process::Command};

#[cfg(all(feature = "static", feature = "dylib"))]
compile_error!("static and dylib are mutually exclusive and cannot be enabled together");

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

    // Get the AOT SDK path
    let sdk_path = nuget_path
        .join("runtime.win-x64.microsoft.dotnet.ilcompiler")
        .join("7.0.0")
        .join("sdk");

    // Change nuget packages path
    unsafe {
        env::set_var("NUGET_PACKAGES", nuget_path.clone());
    }

    // Re-run build script if the bridge code changes
    println!(
        "cargo:rerun-if-changed={}",
        project_path.join("src/FFI.cs").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        project_path.join("lhm-bridge.csproj").display()
    );

    let mut command = Command::new("dotnet");
    command
        // Run the publish command for the bridge project
        .arg("publish")
        .arg(project_path.join("lhm-bridge.csproj"))
        // Select build configuration
        .arg("-c")
        .arg("Release")
        // Select runtime target
        .arg("-r")
        .arg("win-x64")
        // Setup output directory
        .arg("-o")
        .arg(&publish_path)
        // Force the OutputPath and BaseIntermediateOutputPath to be contained withing
        // OUT_DIR because build scripts are not supposed to write outside this dir
        .arg(format!("/p:OutputPath={}/", publish_path.display()))
        .arg(format!(
            "/p:BaseIntermediateOutputPath={}/",
            intermediate_path.display()
        ));

    // Build a shared .dll library
    #[cfg(feature = "dylib")]
    command.arg("/p:NativeLib=Shared");

    // Build a static .lib library
    #[cfg(feature = "static")]
    command.arg("/p:NativeLib=Static");

    let status = command
        .status()
        .expect("failed to run dotnet publish command");

    // Build the C# project
    if !status.success() {
        panic!("failed to build binding library");
    }

    let native_path = publish_path.join("native");

    // Setup linker paths
    println!("cargo:rustc-link-search=native={}", native_path.display());
    println!("cargo:rustc-link-search=native={}", sdk_path.display());

    #[cfg(feature = "dylib")]
    link_dylib();

    #[cfg(feature = "static")]
    link_static();
}

#[cfg(feature = "dylib")]
fn link_dylib() {
    // Link to bridge library
    println!("cargo:rustc-link-lib=dylib=lhm-bridge");
}

#[cfg(feature = "static")]
fn link_static() {
    // Link to required C# AOT libraries
    println!("cargo:rustc-link-lib=static=bootstrapperdll");
    println!("cargo:rustc-link-lib=static=Runtime.WorkstationGC");
    println!("cargo:rustc-link-lib=static=System.Globalization.Native.Aot");
    println!("cargo:rustc-link-lib=static=System.IO.Compression.Native.Aot");

    // Link to required system libraries
    println!("cargo:rustc-link-lib=dylib=Iphlpapi");
    println!("cargo:rustc-link-lib=dylib=Crypt32");
    println!("cargo:rustc-link-lib=dylib=advapi32");
    println!("cargo:rustc-link-lib=dylib=bcrypt");
    println!("cargo:rustc-link-lib=dylib=ole32");
    println!("cargo:rustc-link-lib=dylib=oleaut32");

    // Link to bridge library
    println!("cargo:rustc-link-lib=static=lhm-bridge");
}
