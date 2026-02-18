use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn build(src_dir: &Path, cache: &str) {
    let project_dir = src_dir.to_string_lossy();
    let pkgbuild_path = src_dir.join("PKGBUILD");

    make(&pkgbuild_path);
}

fn parse(content: &str) -> HashMap<String, String> {
    let mut variables = HashMap::new();

    let re = Regex::new(r#"(?m)^(?P<key>\w+)=["']?(?P<value>[^"'\n#]+)["']?"#).unwrap();

    for caps in re.captures_iter(content) {
        let key = caps.name("key").unwrap().as_str().to_string();
        let value = caps.name("value").unwrap().as_str().trim().to_string();

        variables.insert(key, value);
    }

    variables
}

fn make(pkgbuild_path: &Path) {
    let content = fs::read_to_string(pkgbuild_path).unwrap();

    let vars = parse(&content);
    let pkgname = vars.get("pkgname").map(|s| s.as_str()).unwrap_or("null");
    let pkgver = vars.get("pkgver").map(|s| s.as_str()).unwrap_or("1.0.0");

    println!("{pkgname}");
}
