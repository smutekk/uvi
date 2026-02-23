//TODO: allow for newline in source url

use crate::{download, run_command};
use regex::Regex;
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Default)]
struct ParseResult {
    url: HashMap<String, String>,
    variables: HashMap<String, String>,
    functions: HashMap<String, String>,
}

pub fn build(src_dir: &Path) {
    let pkgbuild_path = src_dir.join("PKGBUILD");

    make(&pkgbuild_path, &src_dir);
}

fn parse(content: &str) -> ParseResult {
    let mut result = ParseResult::default();

    // let url_re = Regex::new(r"(?s)source=\(.*?::(?P<url>[^ \)'\s]+)").unwrap();
    // result.url = url_re.captures(content).map(|caps| {
    //     caps["url"]
    //         .trim_matches(|c| c == '"' || c == '\'')
    //         .to_string()
    // });

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

    let buildfn: &str = result.functions.get("build").unwrap().as_str();
    let formatted_buildfn = buildfn
        .replace("$srcdir/", src_dir_str)
        .replace("$pkgver", pkgver)
        .replace("$pkgname", pkgname)
        .replace("$_pkgname", _pkgname)
        .replace("$pkgbase", pkgbase);

    let url: &str = result.functions.get("source").unwrap().as_str();
    println!("{url}");

    // let formatted_url = url.replace("${pkgver}", pkgver);
    // let formatted_name = formatted_url.rsplit_once("/").unwrap().1;

    // let formatted_path = src_dir.join(formatted_name);

    // match download(&formatted_url, &formatted_path) {
    //     Ok(_meow) => {
    //         println!("=> \x1b[32;1mRunning build() function!\x1b[0m");
    //         run_command(src_dir_str, "bash", &["-c", &formatted_buildfn]);
    //     }
    //     Err(err) => print!("=> \x1b[31;1mFailed to download: {err}\x1b[0m"),
    // };
}
