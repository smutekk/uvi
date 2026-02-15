use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn build(src_dir: &Path, cache: &str) {
    let project_dir = src_dir.to_string_lossy();
    let pkgbuild_path = src_dir.join("PKGBUILD");

    make(&pkgbuild_path);
}

fn make(pkgbuild_path: &Path) {
    let content = fs::read_to_string(pkgbuild_path).unwrap();
    println!("{}", content);
}
