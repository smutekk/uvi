// TODO: remove unzipping depends, just use command

use clap::Parser;
use git2::Repository;
use reqwest::blocking;
use std::env;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
// git_repo requires repo and url to be passed in, change it to just url and seperate using split()

pub mod compilers;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Global package manager (Used on Uvite. Requires Make, CMake, Meson, and Ninja.)",
    override_usage = "uvi <NAME> [OPTIONS]"
)]
struct Args {
    /// Name of package
    name: String,

    /// Uninstall package
    #[arg(long)]
    uninstall: bool,

    /// Build args. Format inside of quotemarks: bargs "arg1 arg2 arg3"
    #[arg(long)]
    bargs: Option<String>,

    /// Usable on Uvite
    #[arg(long)]
    user: bool,

    /// Prefix for installing files
    #[arg(long, default_value = "/usr")]
    prefix: String,

    /// Like --noconfirm from pacman
    #[arg(long)]
    fast: bool, // essentially --noconfirm

    /// Allows for building of requested package, enabled by default.
    #[arg(long)]
    build: bool,

    /// Specify after the name if you set the name to a link
    #[arg(long)]
    link: bool,

    /// If you want to use systemd or not
    #[arg(long)]
    systemd: bool,

    /// Repo to search
    #[arg(long, default_value = "https://aur.archlinux.org/")]
    repo: String,
}

pub fn fetch_env(target_env: &str) -> PathBuf {
    let home_path = match env::var("HOME") {
        Ok(p) => PathBuf::from(p),
        Err(_) => {
            println!("HOME not found, defaulting to tmp");
            PathBuf::from("/tmp")
        }
    };

    let cache = Path::new(&home_path).join(".cache").join("uvi");

    let current = match target_env {
        "HOME" => home_path,
        "CACHE" => cache,
        _ => cache,
    };

    current
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut target_destination = Path::new(&args.prefix);
    let filename = args
        .name
        .as_str()
        .split('/')
        .last()
        .unwrap_or("download.tmp");

    let cache = fetch_env("CACHE");
    let home_path = fetch_env("HOME");

    let file_path = cache.join(filename);

    let query = args.name.as_str();
    let repo = args.repo.as_str();

    if query.ends_with(".git") {
        git_repo(
            query,
            Path::new(&file_path),
            target_destination,
            &cache,
            &repo,
        )?;
    }
    if args.user {
        target_destination = Path::new(&home_path);
    }
    if args.link {
        download(query, Path::new(&file_path))?;
    } else {
        println!("=> Package to download is: {}", query);
        println!("=> Cache path: {:?}", cache);

        if repo == "https://aur.archlinux.org/" {
            git_repo(
                &format!("{repo}{query}.git"),
                Path::new(&file_path),
                target_destination,
                &cache,
                &repo,
            )?;
        }
    }

    // Uninstall handling
    if args.uninstall {
        println!("=> Uninstalling: {query}");
        // use pkg-config to find installed package and then uninstall
    }

    println!("=> Home dir is: {:?}", home_path);
    println!("=> Prefix is: {}", target_destination.display());

    Ok(())
}

pub fn download(url: &str, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let response = blocking::get(url)?;
    let mut dest = File::create(destination)?;
    let content = response.bytes()?;

    copy(&mut content.as_ref(), &mut dest)?;
    drop(dest);

    println!("=> \x1b[32;1mDownloaded file successfully!\x1b[0m");
    println!("{:?}", destination.parent().unwrap());

    unpack(destination, destination.parent().unwrap());

    Ok(())
}

fn git_repo(
    url: &str,
    destination: &Path,
    prefix: &Path,
    cache: &Path,
    repo: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("=> \x1b[33mTesting if repo exists..\x1b[0m");

    println!("=> Set repo: {}", &repo);
    println!("=> Set url: {}", &url);

    let git_pkg_name = &url.rsplit_once("/").unwrap().1;
    let pkg_name = git_pkg_name.rsplit_once(".").unwrap().0;

    println!("=> \x1b[1mPackage name: {pkg_name}\x1b[0m");

    let formatted_url = format!("{repo}packages/{pkg_name}"); // not finding anything in aur
    let url_status = blocking::get(&formatted_url)?; // same thing as 

    if !url_status.error_for_status().is_ok() {
        // Make it so that it doesn't loop when retrying
        println!(
            "=> \x1b[31;1mPackage not found in repo:\x1b[0m {} \n\x1b[33;1mTrying backup repo..\x1b[0m (https://archlinux.org/)",
            repo
        );
        let formatted_url =
            format!("https://gitlab.archlinux.org/archlinux/packaging/packages/{pkg_name}.git");

        git_repo(
            &formatted_url,
            destination,
            prefix,
            &cache,
            "https://gitlab.archlinux.org/archlinux/packaging/",
        )?;
    } else {
        println!("=> \x1b[32;1mUrl returned OK!\x1b[0m");

        println!(
            "=> \x1b[33;1mCloning {} into {}..\x1b[0m",
            url,
            destination.to_string_lossy()
        );

        match Repository::clone(url, destination) {
            Ok(repo) => {
                let mut dir_work = repo.workdir();
                let repo_path = dir_work.get_or_insert_with(|| Path::new("/tmp"));

                println!("=> \x1b[32;1mSucessfully cloned: {:?}\x1b[0m", repo_path);

                install(&repo_path)?;
            }
            Err(e) => panic!("=> Failed to clone: {}", e),
        };
    }
    Ok(())
}

pub fn unpack(file_to_unpack: &Path, destination: &Path) {
    // let file = File::open(file_to_unpack);

    if file_to_unpack
        .extension()
        .map_or(false, |ext| ext == "gz" || ext == "xz")
    {
        println!("=> \x1b[33;1mTar detected, unzipping with xzvf..\x1b[0;m");

        Command::new("tar")
            .current_dir(destination)
            .arg("-xzvf")
            .arg(file_to_unpack)
            // .stdout(Stdio::inherit())
            // .stderr(Stdio::inherit())
            .status()
            .expect("Failed to unzip");
    } else if file_to_unpack.extension().map_or(false, |ext| ext == "zst") {
        println!("ugh")
    }
}

fn install(destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let _dest_str = destination.to_str();
    let inst_str = "/usr";

    let args = Args::parse();
    let build_args = format!("{:?}", args.bargs);

    let mut buildable = false;

    if args.build {
        buildable = true
    }

    let build_dir = destination.join("build");

    if destination.join("meson.build").exists() {
        if destination.join("Makefile").exists() {
            println!("=> Found a Makefile when meson.build exists.. \n Defaulting to meson..");
        }
        println!("=> Building with meson..");

        println!("=> Build directory is: {}", build_dir.to_string_lossy());

        if buildable {
            compilers::meson::build(
                destination.to_str().unwrap(),
                &build_dir.to_string_lossy(),
                &format!("--prefix=/{inst_str} {build_args}"), // not instal_dir/prefix yet
            );
        } else {
            println!("Buildable flag disabled..");
        }
    } else if destination.join("Makefile").exists() && !destination.join("meson.build").exists() {
        println!("=> \x1b[31mFound a Makefile, building with make..\x1b[0m");

        if buildable {
            compilers::make::build(
                destination.to_str().unwrap(),
                &build_dir.to_string_lossy(),
                &format!("--prefix=/{inst_str} {build_args}"),
            );
        } else {
            println!("=> Buildable flag disabled..")
        }

        println!("=> done?")
    } else if destination.join("PKGBUILD").exists() {
        println!("=> Found PKGBUILD, building with makepkg..");

        compilers::pkgbuild::build(destination);
    } else {
        println!("=> No supported build files found, exiting..");
    }

    println!("=> Finished installing, installed to: {inst_str}");

    Ok(())
}

// // fn deb() {}

// // fn rpm() {}
