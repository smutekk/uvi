use clap::Parser;
use git2::Repository;
use reqwest::StatusCode;
use reqwest::blocking;
use std::env;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use tar::Archive;
use xz2::read::XzDecoder;
// use zstd::stream;
use bzip2::read::BzDecoder;

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

    /// Format inside of quotemarks: "--arg1 --arg2 --arg3"
    #[arg(long)]
    args: Option<String>,

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut target_destination = Path::new(&args.prefix);
    let filename = args
        .name
        .as_str()
        .split('/')
        .last()
        .unwrap_or("download.tmp");
    let home_path = match env::var("HOME") {
        Ok(p) => PathBuf::from(p),
        Err(_) => {
            println!("HOME not found, defaulting to tmp");
            PathBuf::from("/tmp")
        }
    };

    let cache = Path::new(&home_path).join(".cache").join("uvi");
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
    } else {
        println!("Downloading: {query}");
    }
    if args.user {
        target_destination = Path::new(&home_path);
    }
    if args.link {
        println!("Using url: {}", query);
        download(
            query,
            Path::new(&file_path),
            Path::new(&cache),
            target_destination,
        )?;
    } else {
        println!("Package to download is: {}", query);

        println!("{}", format!("Using url: {repo}{query}"));

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

    println!("Home dir is: {:?}", home_path);
    println!("Prefix is: {}", target_destination.display());

    Ok(())
}

pub fn sudo(dir: &Path, args: &[&str]) -> Result<bool, Box<dyn std::error::Error>> {
    let status = Command::new("sudo")
        .current_dir(dir)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    Ok(status.success())
}

fn download(
    url: &str,
    destination: &Path,
    cache: &Path,
    prefix: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = blocking::get(url)?;
    let mut dest = File::create(destination)?;
    let content = response.bytes()?;

    copy(&mut content.as_ref(), &mut dest)?;
    drop(dest);

    unpack(Path::new(destination), &cache, prefix)?;

    Ok(())
}

fn git_repo(
    url: &str,
    destination: &Path,
    prefix: &Path,
    cache: &Path,
    repo: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing if repo exists..");
    let git_pkg_name = url.rsplit_once("/").unwrap().1;
    let pkg_name = git_pkg_name.rsplit_once(".").unwrap().0;
    let formatted_url = format!("{repo}packages/{pkg_name}");
    let url_status = blocking::get(formatted_url)?;

    println!("{}", pkg_name);

    if !url_status.error_for_status().is_ok() {
        println!("Package not found in repo: {}", repo)
    } else {
        println!("Cloning {} into {:?}..", url, destination);

        match Repository::clone(url, destination) {
            Ok(repo) => {
                let mut dir_work = repo.workdir();
                let repo_path = dir_work.get_or_insert_with(|| Path::new("/tmp"));

                println!("Sucessfully cloned: {:?}", repo_path);

                install(&repo_path, prefix, cache)?;
            }
            Err(e) => panic!("Failed to clone: {}", e),
        };
    }
    Ok(())
}

fn unpack(
    file_to_unpack: &Path,
    destination: &Path,
    prefix: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let file = File::open(file_to_unpack)?;

    if file_to_unpack.extension().map_or(false, |ext| ext == "xz") {
        println!("XZ file detected! starting unpack process..");

        let decompressor = XzDecoder::new(file);
        let mut archive = Archive::new(decompressor);

        archive.unpack(destination)?;
    } else if file_to_unpack.extension().map_or(false, |ext| ext == "zst") {
        println!("ZST file detected! starting unpack process..");

        // stream::copy_decode(file, destination)?; //not working
    } else if file_to_unpack.extension().map_or(false, |ext| ext == "bz") {
        println!("BZ file detected! starting unpack proces..");

        let _decompressor = BzDecoder::new(file);

        //TODO
    }

    let unpacked_file = file_to_unpack
        .file_stem()
        .and_then(|s| Path::new(s).file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("default");

    install(
        &destination.join(unpacked_file.to_string()),
        prefix,
        Path::new(""),
    )?;
    // return Ok(destination.to_path_buf());

    Err("File ext. not supported.".into())
}

fn install(
    destination: &Path,
    install_dir: &Path,
    cache: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let dest_str = destination.to_str();
    let inst_str = install_dir.to_string_lossy();

    let args = Args::parse();
    let build_args = format!("{:?}", args.args);

    let mut buildable = false;

    if args.build {
        buildable = true
    }

    println!("unpacked file located at: {}", dest_str.unwrap());

    let build_dir = destination.join("build");

    if destination.join("meson.build").exists() {
        if destination.join("Makefile").exists() {
            println!("Found a Makefile when meson.build exists.. \n Defaulting to meson..");
        }
        println!("Building with meson..");

        println!("Build directory is: {}", build_dir.to_string_lossy());

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
        println!("Found a Makefile, building with make..");

        if buildable {
            compilers::make::build(
                destination.to_str().unwrap(),
                &build_dir.to_string_lossy(),
                &format!("--prefix=/{inst_str} {build_args}"),
            );
        } else {
            println!("Buildable flag disabled..")
        }

        println!("done?")
    } else if destination.join("PKGBUILD").exists() {
        println!("Found PKGBUILD, building with makepkg..");

        compilers::pkgbuild::build(destination, cache.to_str().unwrap(), &build_args)?;
    } else {
        println!("No supported build files found, exiting..");
    }

    println!("Finished installing, installed to: {inst_str}");

    Ok(())
}

// // fn deb() {}

// // fn rpm() {}
