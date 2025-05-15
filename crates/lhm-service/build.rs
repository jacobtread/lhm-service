fn main() {
    println!("cargo:rustc-flags=-l ole32");
    println!("cargo:rustc-link-lib=dylib=Iphlpapi");
    println!("cargo:rustc-link-lib=dylib=bcrypt");
    println!("cargo:rustc-link-lib=dylib=OleAut32");
    println!("cargo:rustc-link-lib=dylib=Crypt32");
}
