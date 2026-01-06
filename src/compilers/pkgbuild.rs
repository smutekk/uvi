use regex::Regex;
use reqwest::blocking;
use std::fs;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn build(project_dir: &str, arguments: &str, cache: &str) {
    make(project_dir, arguments, cache)
}

fn make(project_dir: &str, prefix: &str, cache: &str) {
    let args = vec!["--i", "--noconfirm"];
    run_command(project_dir, "makepkg", &args, cache);
}

fn run_command(dir: &str, name: &str, args: &[&str], cache: &str) {
    let status = Command::new(name)
        .current_dir(dir)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to launch the process"); // Could not resolve all dependencies

    if !status.success() {
        let deps = get_deps(dir);

        println!("Detected dependencies from MAKEPKG:");
        for dep in deps {
            download_deps(&dep, cache);
            println!(" - {}", dep);
        }
        eprintln!("Downloading package's dependencies");

        // make();
    }
}

fn download_deps(dependency: &str, cache: &str) {
    // let dep_name = dependency

    let response = blocking::get(format!("https://aur.archlinux.org/packages/{dependency}"));
    let mut dest = File::create(&cache);
    let content = response.bytes();

    copy(&mut content.as_ref(), &mut dest);
    drop(dest);
}

fn get_deps(project_dir: &str) -> Vec<String> {
    let pkgbuild_path = Path::new(project_dir).join("meson.build");

    let content = match fs::read_to_string(&pkgbuild_path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Could not read PKGBUILD from: {:?}", pkgbuild_path);
            return vec![];
        }
    };

    let re = Regex::new(r#"depends\s*\(\s*['"]([^'"]+)['"]"#).unwrap();

    let mut deps: Vec<String> = re
        .captures_iter(&content)
        .map(|cap| cap[1].to_string())
        .collect();

    deps.sort();
    deps.dedup();
    deps
}
