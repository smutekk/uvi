use std::process::{Command, Stdio};

pub fn build(project_dir: &str, build_dir: &str, arguments: &str) {
    install(project_dir, build_dir, arguments);
}

fn install(project_dir: &str, build_dir: &str, user_args: &str) {
    let args = vec!["--prefix=/usr"];

    run_command(project_dir, "./configure", &args);
    run_command(project_dir, "make", &vec!["-m"]);
    run_command(project_dir, "make", &vec!["install"]);
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
        eprintln!("Starting download process..");

        std::process::exit(1);
    }
}
