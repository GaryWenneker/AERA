use std::{env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let assets = root.join("assets/wallpapers");
    println!("cargo:rerun-if-changed={}", assets.display());
    let mut code = String::from("pub const PHOTOS: [Option<&[u8]>; 3] = [\n");
    for name in ["alpine.jpg", "midnight.jpg", "autumn.jpg"] {
        let path = assets.join(name);
        if path.is_file() {
            code.push_str(&format!("Some(include_bytes!({:?})),\n", path));
        } else {
            code.push_str("None,\n");
        }
    }
    code.push_str("];\n");
    fs::write(out.join("photos.rs"), code).unwrap();
}
