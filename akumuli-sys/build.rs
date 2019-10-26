fn main() {
    //println!("cargo:rustc-link-lib=static=apr-1.0");
    //println!("cargo:rustc-link-search=native=/usr/include");

    let bindings = bindgen::Builder::default()
        .header("akumuli/include/akumuli.h")
        .clang_arg("-I/usr/include/apr-1.0")
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
