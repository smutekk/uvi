//TODO: allow for newline in source url

use crate::download;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Default)]
struct ParseResult {
    url: Option<String>,
    variables: HashMap<String, String>,
    functions: HashMap<String, String>,
}

pub fn build(src_dir: &Path) {
    let pkgbuild_path = src_dir.join("PKGBUILD");

    make(&pkgbuild_path, &src_dir);
}

fn parse(content: &str) -> ParseResult {
    let mut result = ParseResult::default();

    let url_re = Regex::new(r"(?s)source=\(.*?::(?P<url>[^ \)'\s]+)").unwrap();
    result.url = url_re.captures(content).map(|caps| {
        caps["url"]
            .trim_matches(|c| c == '"' || c == '\'')
            .to_string()
    });

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

    let result = parse(&content);
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

    if let Some(url) = result.url {
        let formatted_url = url.replace("${pkgver}", pkgver);
        let formatted_name = formatted_url.rsplit_once("/").unwrap().1;

        let formatted_path = src_dir.join(formatted_name);

        download(&formatted_url, &formatted_path)
            .expect("Failed to download, possible incorrrect url/path?");

        //TODO:MAKE SURE BUILD THING WORKS
    } else {
        eprintln!("=> \x1b[31;mERROR: source url not found?\x1b[0m");
    };
}
