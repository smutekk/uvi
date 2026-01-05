use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn build(project_dir: &str, build_dir: &str, arguments: &str) {
    run_meson(project_dir, build_dir, arguments);
}

fn run_meson(lib: &str, dir: &str, arguments: &str) {
    if !is_configured(lib) {
        let mut args = vec!["setup", ".", dir];
        let extra_args: Vec<&str> = arguments.split_whitespace().collect();

        args.extend(extra_args);

        run_command(lib, "meson", &args);
    } else {
        println!("Configured successfully! \n Now building..");
    }
}

fn run_command(dir: &str, name: &str, args: &[&str]) {
    let status = Command::new(name)
        .current_dir(dir)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to launch the process");

    if !status.success() {
        let deps = get_deps(dir);

        println!("Detected dependencies from meson.build:");
        for dep in deps {
            println!(" - {}", dep);
        }
        eprintln!("Starting download process..");

        std::process::exit(1);
    }
}

fn get_deps(project_dir: &str) -> Vec<String> {
    let meson_path = Path::new(project_dir).join("meson.build");

    let content = match fs::read_to_string(&meson_path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Could not read meson.build at {:?}", meson_path);
            return vec![];
        }
    };

    let re = Regex::new(r#"dependency\s*\(\s*['"]([^'"]+)['"]"#).unwrap();

    let mut deps: Vec<String> = re
        .captures_iter(&content)
        .map(|cap| cap[1].to_string())
        .collect();

    deps.sort();
    deps.dedup();
    deps
}

fn is_configured(dir: &str) -> bool {
    let mut path = PathBuf::from(dir);
    path.push("build.ninja");
    return path.as_path().exists();
}
