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
// use zstd::stream;
use bzip2::read::BzDecoder;

mod compilers;

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
    prefix: String,

    #[arg(long)]
    fast: bool, // essentially --noconfirm

    #[arg(long)]
    link: bool,

    #[arg(long)]
    systemd: bool,

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

    let cache = Path::new(&home_path).join(".cache");
    let file_path = cache.join(filename);

    let query = args.name.as_str();
    let repo = args.repo.as_str();

    if query.ends_with(".git") {
        git_repo(query, Path::new(&file_path), target_destination, &cache)?;
    } else {
        println!("Downloading: {query}");
    }
    if args.user {
        target_destination = Path::new(&home_path); //get username
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
            )?;
        }

        // download(
        //     &format!("repo{query}"),
        //     &file_path,
        //     &cache,
        //     target_destination,
        // )?;
    }

    println!("Home dir is: {:?}", home_path);
    println!("Prefix is: {}", target_destination.display());

    Ok(())
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
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Cloning {} into {:?}..", url, destination);

    match Repository::clone(url, destination) {
        Ok(repo) => {
            let mut dir_work = repo.workdir();
            let repo_path = dir_work.get_or_insert_with(|| Path::new("/tmp"));

            println!("Sucessfully cloned: {:?}", repo_path);

            install(&repo_path, prefix, cache);
        }
        Err(e) => panic!("Failed to clone: {}", e),
    };

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
    );
    // return Ok(destination.to_path_buf());

    Err("File ext. not supported.".into())
}

fn install(destination: &Path, install_dir: &Path, cache: &Path) {
    let dest_str = destination.to_str();
    let inst_str = install_dir.to_string_lossy();

    println!("unpacked file located at: {}", dest_str.unwrap());

    let build_dir = destination.join("build");

    if destination.join("meson.build").exists() {
        if destination.join("Makefile").exists() {
            println!("Found a Makefile when meson.build exists.. \n Defaulting to meson..");
        }
        println!("Building with meson..");

        println!("Build directory is: {}", build_dir.to_string_lossy());

        compilers::meson::build(
            destination.to_str().unwrap(),
            &build_dir.to_string_lossy(),
            &format!("--prefix=/{inst_str}"), // not instal_dir/prefix yet
        );
    } else if destination.join("Makefile").exists() && !destination.join("meson.build").exists() {
        println!("Found a Makefile, building with make..");

        compilers::make::build(
            destination.to_str().unwrap(),
            &build_dir.to_string_lossy(),
            &format!("--prefix=/{inst_str}"),
        );

        println!("done?")
    } else if destination.join("PKGBUILD").exists() {
        println!("Found PKGBUILD, building with makepkg..");

        compilers::pkgbuild::build(destination.to_str().unwrap(), cache.to_str().unwrap());
    } else {
        println!("No supported build files found, exiting..");
    }

    println!("Finished installing, installed to: {inst_str}");
}

// // fn deb() {}

// // fn rpm() {}
