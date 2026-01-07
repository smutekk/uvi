use regex::Regex;
use reqwest::blocking;
use std::fs;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn build(project_dir: &str, cache: &str) {
    println!("Building..");

    get_deps(project_dir);
    make(project_dir, cache);
}

fn make(project_dir: &str, cache: &str) {
    let args = vec!["--install", "--noconfirm"];
    // let deps = get_deps(project_dir);

    // for dep in deps {
    //     // download_deps(&dep, cache)?;
    //     println!(" - {}", dep);
    // }

    run_command(project_dir, "makepkg", &args, cache);
}

fn run_command(dir: &str, name: &str, args: &[&str], _cache: &str) {
    let status = Command::new(name)
        .current_dir(dir)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to launch the process"); // Could not resolve all dependencies

    if !status.success() {
        // let deps = get_deps(dir);

        // println!("Detected dependencies from MAKEPKG:");
        // eprintln!("Downloading package's dependencies");

        // make();
    }
}

fn download_deps(dependency: &str, cache: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = blocking::get(format!("https://aur.archlinux.org/packages/{dependency}"))?;
    let mut dest = File::create(&cache)?;
    let content = response.bytes()?;
    let dest_path = Path::new(&cache).join(format!("{dest:?}"));

    println!("{:?}", dest_path);
    copy(&mut content.as_ref(), &mut dest)?;
    drop(dest);

    // unpack(&, &cache);

    Ok(())
}

// fn unpack(target: &Path, destination: &Path) {
//     zstd::stream::copy_decode(target, destination);
// }

fn get_deps(project_dir: &str) {
    println!("Getting deps..");
    let pkgbuild_path = Path::new(project_dir).join("PKGBUILD");

    let content = match fs::read_to_string(&pkgbuild_path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Could not read PKGBUILD from: {:?}", pkgbuild_path);
            return;
        }
    };
    println!("Read PKGBUILD");

    // let mut dependencies = vec![];

    let re = Regex::new(r#"[-()>='".0-9]|depends|make"#).unwrap();

    println!(
        "{:?}",
        re.captures(&content).map(|cap| println!("{:?}", cap))
    );
    // let deps = re.captures(&content);
    // .map(|cap| cap[1].to_string())

    // println!("{:?}", deps);
    println!("Got dependencies!");

    // println!("{}", deps)

    // deps.sort();
    // deps.dedup();
    // deps
}
