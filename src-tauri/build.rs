use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if let Some(bin_dir) = find_ortools_bin_dir() {
        println!("cargo:rerun-if-changed={}", bin_dir.display());
        if let Some(target_dir) = target_profile_dir() {
            let copy_dir = target_dir.join("ortools");
            if let Err(error) = copy_runtime_dir(&bin_dir, &copy_dir) {
                panic!(
                    "failed to copy OR-Tools runtime to {}: {error}",
                    copy_dir.display()
                );
            }
            println!(
                "cargo:rustc-env=ACADEMIC_ORTOOLS_DEV_DIR={}",
                copy_dir.display()
            );
        }
    }
    tauri_build::build()
}

fn find_ortools_bin_dir() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").ok()?);
    let vendor_root = manifest_dir.join("vendor");
    let entries = fs::read_dir(vendor_root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let bin_dir = path.join("bin");
        if bin_dir.join(exe_name("sat_runner")).is_file() {
            return Some(bin_dir);
        }
    }
    None
}

fn target_profile_dir() -> Option<PathBuf> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").ok()?);
    out_dir.parent()?.parent()?.parent().map(Path::to_path_buf)
}

fn copy_runtime_dir(source: &Path, target: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let file_name = entry.file_name();
        fs::copy(&path, target.join(file_name))?;
    }
    Ok(())
}

fn exe_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}
