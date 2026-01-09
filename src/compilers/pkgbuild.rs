use git2::Repository;
use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn build(src_dir: &Path, cache: &str) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = src_dir.to_string_lossy();

    make(&project_dir, cache)?;
    install(src_dir, "/")?; // TODO: prefix
    Ok(())
}

pub fn sudo(dir: &str, args: &[&str]) -> Result<bool, Box<dyn std::error::Error>> {
    let dir_path = Path::new(dir);
    let status = Command::new("sudo")
        .current_dir(dir_path)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    Ok(status.success())
}

fn make(project_dir: &str, cache: &str) -> Result<(), Box<dyn std::error::Error>> {
    let args = vec!["--noconfirm", "--needed"];
    if run(project_dir, "makepkg", &args)? {
        return Ok(());
    }

    let deps = get_deps(project_dir);

    for dep in deps {
        let url = format!("https://aur.archlinux.org/{dep}.git");
        if !aur_has_refs(&url) {
            continue;
        }

        let path = Path::new(cache).join(&dep);
        if !path.exists() {
            Repository::clone(&url, &path)?;
        }

        run(path.to_str().unwrap(), "makepkg", &args)?;
    }

    run(project_dir, "makepkg", &args)?;
    Ok(())
}

fn install(src: &Path, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Installing package to: {}", dest);
    let src_str = src.to_string_lossy();

    let build_dir = Path::new(src).join("pkg");
    // let build_str = build_dir.to_string_lossy();
    let build_name = src
        .file_name()
        .unwrap()
        .to_string_lossy()
        .replace("\"", "")
        .to_string();

    println!("{}", build_name);
    let install_dir = Path::new(&build_dir).join(build_name);
    println!("{:?}", install_dir);
    let install_str = format!("{}", install_dir.to_string_lossy());
    // let joined_str = format!("{build_str}/{build_name}/*");
    println!("{}", install_str);
    for component in fs::read_dir(install_str).unwrap() {
        let component_str = format!("{:?}", component.unwrap().file_name());
        let component_path = install_dir.join(&component_str);
        let component_path_str = component_path
            .to_string_lossy()
            .replace("\"", "")
            .to_string();
        println!("{}", component_path_str);
        let args = vec!["cp", "-ivr", &component_path_str, &dest];

        sudo(&src_str, &args)?;
    }

    Ok(())
}

fn run(dir: &str, cmd: &str, args: &[&str]) -> Result<bool, Box<dyn std::error::Error>> {
    let status = Command::new(cmd)
        .current_dir(dir)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    Ok(status.success())
}

fn aur_has_refs(url: &str) -> bool {
    Command::new("git")
        .args(["ls-remote", url])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

fn get_deps(project_dir: &str) -> Vec<String> {
    let pkgbuild_path = Path::new(project_dir).join("PKGBUILD");
    let content = match fs::read_to_string(pkgbuild_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let re = Regex::new(r#"(?m)^(?:make)?depends=\(([\s\S]*?)\)"#).unwrap();
    let mut deps = Vec::new();

    for cap in re.captures_iter(&content) {
        for dep in cap[1].split_whitespace() {
            let d = dep
                .trim_matches(|c| c == '\'' || c == '"' || c == '(' || c == ')')
                .split(&['=', '>', '<'][..])
                .next()
                .unwrap();
            if !d.is_empty() {
                deps.push(d.to_string());
            }
        }
    }

    deps.sort();
    deps.dedup();
    deps
}
