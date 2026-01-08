use git2::Repository;
use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn build(project_dir: &str, cache: &str) -> Result<(), Box<dyn std::error::Error>> {
    make(project_dir, cache)
}

fn make(project_dir: &str, cache: &str) -> Result<(), Box<dyn std::error::Error>> {
    let args = vec!["--install", "--noconfirm", "--needed"];
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
