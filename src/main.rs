use clap::Parser;
use git2::Repository;
use reqwest::blocking;
use std::env;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::path::PathBuf;
use tar::Archive;
use xz2::read::XzDecoder;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Global package manager (Used on Uvite. Requires Make, CMake, Meson, and Ninja.)",
    override_usage = "uvi <NAME> [OPTIONS]"
)]
struct Args {
    name: String,

    #[arg(long)]
    user: bool,

    #[arg(long, default_value = "/usr")]
    destination: String,

    #[arg(long)]
    fast: bool, // essentially --noconfirm

    #[arg(long)]
    link: bool,

    #[arg(long)]
    systemd: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut target_destination = Path::new(&args.destination);
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

    let cache = Path::new(&home_path).join(".cache");
    let file_path = cache.join(filename);

    if args.user {
        target_destination = Path::new(&home_path); //get username
    }

    println!("Home dir is: {:?}", home_path);
    println!("Prefix is: {}", target_destination.display());

    if args.link {
        println!("Using url: {}", args.name.as_str());
        if args.name.as_str().ends_with(".git") {
            git_repo(args.name.as_str(), Path::new(&file_path))?;
        } else {
            download(args.name.as_str(), Path::new(&file_path), Path::new(&cache))?;
        }
        // soo easy
    } else {
        println!("Package to download is: {}", args.name.as_str());
        // bit harder
    }

    Ok(())
}

fn download(url: &str, destination: &Path, cache: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let response = blocking::get(url)?;
    let mut dest = File::create(destination)?;
    let content = response.bytes()?;

    copy(&mut content.as_ref(), &mut dest)?;
    drop(dest);

    unpack(Path::new(destination), &cache);

    Ok(())
}

fn git_repo(url: &str, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Cloning {} into {:?}..", url, destination);

    match Repository::clone(url, destination) {
        Ok(repo) => println!("Sucessfully cloned: {:?}", repo.path()),
        Err(e) => panic!("Failed to clone: {}", e),
    };

    Ok(())
}

fn unpack(file_to_unpack: &Path, destination: &Path) {
    if file_to_unpack.extension().map_or(false, |ext| ext == "xz") {
        println!("XZ file detected! starting unpack process..");

        let file = match File::open(file_to_unpack) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open file: {}", e);
                return;
            }
        };

        let decompressor = XzDecoder::new(file);
        let mut archive = Archive::new(decompressor);

        archive.unpack(destination).expect("Failed");
    }
}
// // fn deb() {}

// // fn rpm() {}

// fn git(destination: &Path) {}
