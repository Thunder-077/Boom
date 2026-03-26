extern crate prost_build;

fn main() {
    if let Some(protoc) = bundled_protoc_path() {
        std::env::set_var("PROTOC", protoc);
    }

    prost_build::compile_protos(
        &["src/cp_model.proto", "src/sat_parameters.proto"],
        &["src/"],
    )
    .unwrap();
}

fn bundled_protoc_path() -> Option<String> {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").ok()?);
    let vendor_root = manifest_dir.parent()?.parent()?.join("vendor");
    let entries = std::fs::read_dir(vendor_root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let protoc = path.join("bin").join(exe_name("protoc"));
        if protoc.is_file() {
            return Some(protoc.to_string_lossy().into_owned());
        }
    }
    None
}

fn exe_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}
