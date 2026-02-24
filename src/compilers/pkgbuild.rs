//TODO: check for +git in source
// TODO: also some PKGBUILDs contain multiple sources??

use crate::{download, run_command};
use regex::Regex;
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Default)]
struct ParseResult {
    url: Option<String>, // TODO: fix this [allow for newline]
    variables: HashMap<String, String>,
    functions: HashMap<String, String>,
}

pub fn build(src_dir: &Path) {
    let pkgbuild_path = src_dir.join("PKGBUILD");

    make(&pkgbuild_path, &src_dir);
}

fn parse(content: &str) -> ParseResult {
    let mut result = ParseResult::default();

    let url_re =
        Regex::new(r#"(?s)source=\(\s*["']?(?:(?P<alias>[^"'\s)]+)::)?(?P<url>[^"'\s)]+)"#)
            .unwrap();

    if let Some(caps) = url_re.captures(content) {
        result.url = Some(caps["url"].to_string());
    }

    let kv_re = Regex::new(r#"(?m)^(?P<key>\w+)=["']?(?P<value>[^"'\n#]+)["']?"#).unwrap(); // god i hate regex
    for caps in kv_re.captures_iter(content) {
        let key = caps["key"].to_string();
        let value = caps["value"].trim().to_string();
        result.variables.insert(key, value);
    }
    let func_re = Regex::new(r"(?m)^(?P<name>\w+)\s*\(\s*\)\s*\{(?P<body>(?s).*?)\n\}").unwrap();
    for caps in func_re.captures_iter(content) {
        let name = caps["name"].to_string();
        let body = caps["body"].trim().to_string();
        result.functions.insert(name, body);
    }

    result
}

fn make(pkgbuild_path: &Path, src_dir: &Path) {
    let content = fs::read_to_string(pkgbuild_path).unwrap();
    let src_dir_str = src_dir.to_str().expect("failed");

    let result = parse(&content);

    let url: &str = &result.url.expect("Failed");

    let pkgname = result
        .variables
        .get("pkgname")
        .map(|s| s.as_str())
        .unwrap_or("null");

    // println!("{url}");

    let build_fn: &str = result.functions.get("build").unwrap().as_str();

    let pkg_fn_name;
    let fmt_name = format!("package_{pkgname}");

    if let Some(pkg) = result.functions.get("package") {
        println!("=> \x1b[32;1mFound package()!\x1b[0m");
        pkg_fn_name = pkg;
    } else {
        println!("=> \x1b[33;1mTrying package_{}()\x1b[0m", pkgname);
        pkg_fn_name = &fmt_name;
    }
    let pkg_fn = result.functions.get(pkg_fn_name).unwrap().as_str(); // TODO; doesnt wor
    // result
    // .functions
    // .get("package")
    // .expect("no pkg(), try pkg_name");
    // .unwrap()
    // .as_str();

    let formatted_url = format_pkgbuild(url, &content, src_dir_str);
    let formatted_pkg_fn = format_pkgbuild(pkg_fn, &content, src_dir_str);
    let formatted_build_fn = format_pkgbuild(build_fn, &content, src_dir_str);

    let formatted_name = formatted_url.rsplit_once("/").unwrap().1;
    let formatted_path = src_dir.join(formatted_name);

    match download(&formatted_url, &formatted_path) {
        Ok(_meow) => {
            println!("=> \x1b[32;1mRunning build() function!\x1b[0m");
            match run_command(src_dir_str, "bash", &["-c", &formatted_build_fn]) {
                Ok(_) => {
                    println!("=> \x1b[31;1mRunning package() function!\x1b[0m");
                    run_command(src_dir_str, "sudo", &["-c", &formatted_pkg_fn])
                        .expect("=> \x1b[31;1mERR: Failed to run command..");
                }
                Err(_e) => println!("Err"),
            };
        }
        Err(err) => println!("=> \x1b[31;1mERR: {err}\x1b[0m"),
    };
}

fn format_pkgbuild(input: &str, content: &str, src_dir_str: &str) -> String {
    let result = parse(&content);

    let _pkgname = result
        .variables
        .get("_pkgname")
        .map(|s| s.as_str())
        .unwrap_or("null");
    let pkgbase = result
        .variables
        .get("pkgbase")
        .map(|s| s.as_str())
        .unwrap_or("null");
    let pkgname = result
        .variables
        .get("pkgname")
        .map(|s| s.as_str())
        .unwrap_or("null");
    let pkgver = result
        .variables
        .get("pkgver")
        .map(|s| s.as_str())
        .unwrap_or("1.0.0");
    let _name = result
        .variables
        .get("_name")
        .map(|s| s.as_str())
        .unwrap_or(pkgname);

    let formatted = input
        .replace("$srcdir/", src_dir_str)
        .replace("$pkgver", pkgver)
        .replace("$pkgname", pkgname)
        .replace("$_pkgname", _pkgname)
        .replace("$pkgbase", pkgbase)
        .replace("$_name", _name)
        .replace("${folder}", "linux-unpacked")
        .replace("${pkgver}", pkgver)
        .replace("${pkgname}", pkgname)
        .replace("{,.sig}", "")
        .replace("git+", "");

    formatted
}
