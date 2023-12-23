//! Build script, used to vendor the frontend if requested.

use std::{
    env::{remove_var, var, vars},
    path::PathBuf,
    process::{Command, Stdio},
};
use walkdir::WalkDir;

fn main() {
    if var("CARGO_FEATURE_FRONTEND_VENDOR").is_err() {
        return;
    }

    let frontend = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap()).join("../frontend");
    let out_dir_files = PathBuf::from(var("OUT_DIR").unwrap()).join("frontend_files.rs");
    let dist = frontend.join("dist");

    // unset all cargo variables for building to succeed
    for (name, _value) in vars() {
        if name.starts_with("CARGO") {
            remove_var(name);
        }
    }

    // launch trunk build
    let status = Command::new("trunk")
        .arg("build")
        .arg("--release")
        .current_dir(&frontend)
        .stdout(Stdio::piped())
        .status()
        .expect("Running trunk to build frontend");

    // make sure building succeeded
    assert!(status.success(), "error building frontend");

    // write out rust file referencing all frontend files
    let mut output = "[".to_string();
    let files = WalkDir::new(&dist)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir());

    for file in files {
        output.push_str(&format!(
            "({:?}, include_bytes!({:?})),",
            file.path().strip_prefix(&dist).unwrap(),
            file.path()
        ));
    }

    output.push(']');

    std::fs::write(out_dir_files, output).unwrap();
}
