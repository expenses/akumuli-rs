fn main() {
    println!("cargo:rustc-link-lib=dylib=apr-1");
    println!("cargo:rustc-link-lib=akumuli");
    println!("rerun-if-changed=build.rs");

    let bindings = bindgen::Builder::default()
        .header("/usr/local/include/akumuli.h")
        .clang_arg("-I/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include/")
        .clang_arg("-I/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include/apr-1")
        .blacklist_type("i64")
        .blacklist_type("i32")
        .blacklist_type("i16")
        .blacklist_type("i8")
        .blacklist_type("u64")
        .blacklist_type("u32")
        .blacklist_type("u16")
        .blacklist_type("u8")
        .generate().unwrap();

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs")).unwrap();
}
