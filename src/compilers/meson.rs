// Edited version of the meson cargo package.

use crate::run_command;
use std::path::PathBuf;

pub fn build(project_dir: &str, build_dir: &str, arguments: &str) {
    run_meson(project_dir, build_dir, arguments);
    run_ninja(project_dir, build_dir);
}

fn run_ninja(project_dir: &str, build_dir: &str) {
    let args = vec!["-C", build_dir];
    let install_args = vec!["-C", build_dir, "install"];
    run_command(project_dir, "ninja", &args).expect("=>\x1b[31m ERR: failed to run ninja");
    run_command(project_dir, "ninja", &install_args)
        .expect("=>\x1b[31m ERR: failed to run ninja [install]");
}

fn run_meson(lib: &str, dir: &str, arguments: &str) {
    if !is_configured(lib) {
        let mut args = vec!["setup", ".", dir];
        let extra_args: Vec<&str> = arguments.split_whitespace().collect();

        args.extend(extra_args);

        run_command(lib, "meson", &args).expect("=>\x1b[31m ERR: failed to run meson");
    }
}

// fn get_deps(project_dir: &str) -> Vec<String> {
//     let meson_path = Path::new(project_dir).join("meson.build");

//     let content = match fs::read_to_string(&meson_path) {
//         Ok(c) => c,
//         Err(_) => {
//             eprintln!("Could not read meson.build at {:?}", meson_path);
//             return vec![];
//         }
//     };

//     let re = Regex::new(r#"dependency\s*\(\s*['"]([^'"]+)['"]"#).unwrap();

//     let mut deps: Vec<String> = re
//         .captures_iter(&content)
//         .map(|cap| cap[1].to_string())
//         .collect();

//     deps.sort();
//     deps.dedup();
//     deps
// }

fn is_configured(dir: &str) -> bool {
    let mut path = PathBuf::from(dir);
    path.push("build.ninja");
    return path.as_path().exists();
}
