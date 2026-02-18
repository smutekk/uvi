//TODO: merge the two parse functions, replace pkgver found in the source url,
// allow for newline in source url

use crate::download;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn build(src_dir: &Path) {
    let project_dir = src_dir.to_string_lossy();
    let pkgbuild_path = src_dir.join("PKGBUILD");

    make(&pkgbuild_path, &src_dir);
}

fn parse_url(content: &str) -> Option<String> {
    let re = Regex::new(r"(?s)source=\(.*?::(?P<url>[^ \)'\s]+)").unwrap();

    re.captures(content).map(|caps| {
        caps["url"]
            .trim_matches(|c| c == '"' || c == '\'')
            .to_string()
    })
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

fn make(pkgbuild_path: &Path, src_dir: &Path) {
    let content = fs::read_to_string(pkgbuild_path).unwrap();

    let vars = parse(&content);
    let pkgname = vars.get("pkgname").map(|s| s.as_str()).unwrap_or("null");
    let pkgver = vars.get("pkgver").map(|s| s.as_str()).unwrap_or("1.0.0");

    if let Some(url) = parse_url(&content) {
        let formatted_url = url.replace("${pkgver}", pkgver);
        let formatted_name = formatted_url.rsplit_once("/").unwrap().1;

        let formatted_path = src_dir.join(formatted_name);

        download(&formatted_url, &formatted_path)
            .expect("Failed to download, possible incorrrect url/path?");

        //TODO: unzip and then extract build and install functions from PKGBUILD
    } else {
        eprintln!("=> \x1b[31;mERROR: source url not found?\x1b[0m");
    };
}
