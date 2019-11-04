use std::env;
use std::path;

fn main() {
    let bindings = bindgen::builder()
        .header("../../godot_headers/gdnative_api_struct.gen.h")
        .clang_arg("-I../../godot_headers")
        .generate()
        .unwrap();
    let out_dir = path::PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_dir.join("bindings.rs")).unwrap();
}
