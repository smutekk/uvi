// TODO: also stop redefining variables / maybe const would work here?
// TODO: work around packages having like 8 names in parenthesis (ghostty ghostty-terminfo)

use crate::{download, run_command};
use regex::Regex;
use std::{collections::HashMap, fs, path::Path};

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

fn download_pkgbuild(pkg_fn_name: &str, url: &str, content: &str, src_dir_str: &str) {
    // TODO: REALLY REALLY BAD

    let result = parse(&content);

    let pkg_error = format!("echo '=> \x1b[1mINFO:\x1b[0m No prepare() function.'");

    let pkg_fn: &str = result.functions.get(pkg_fn_name).unwrap().as_str();
    let build_fn: &str = result.functions.get("build").unwrap().as_str();
    let prepare_fn = result
        .functions
        .get("prepare")
        .unwrap_or(&pkg_error)
        .as_str();

    let formatted_url = format_pkgbuild(url, &content, src_dir_str);
    let formatted_pkg_fn = format_pkgbuild(pkg_fn, &content, src_dir_str);
    let formatted_build_fn = format_pkgbuild(build_fn, &content, src_dir_str);
    let formatted_prepare_fn = format_pkgbuild(prepare_fn, &content, src_dir_str);

    let formatted_name = formatted_url.rsplit_once("/").unwrap().1;
    let formatted_path = Path::new(src_dir_str).join(formatted_name);

    match download(&formatted_url, &formatted_path) {
        Ok(_meow) => {
            // run_command(src_dir_str, "bash", &["-c", &formatted_prepare_fn]).expect("oopsies");
            println!("=> \x1b[32;1mSUC:\x1b[0m Running build() function!");
            match run_command(src_dir_str, "bash", &["-c", &formatted_build_fn]) {
                Ok(_) => {
                    println!("=> \x1b[32;1mSUC:\x1b[0m Running package() function!");
                    run_command(src_dir_str, "sudo", &["bash", "-c", &formatted_pkg_fn])
                        .expect("=> \x1b[31;1mERR: Failed to run command..");
                }
                Err(_e) => println!("Err"),
            };
        }
        Err(err) => println!("=> \x1b[31;1mERR:\x1b[0m {err}"),
    };
}

fn make(pkgbuild_path: &Path, src_dir: &Path) {
    let content = fs::read_to_string(pkgbuild_path).unwrap();
    let src_dir_str = src_dir.to_str().expect("failed");

    let result = parse(&content);

    let url: &str = &result.url.expect("Failed");

    let pkgname: &str = result
        .variables
        .get("pkgname")
        .map(|s| s.as_str())
        .unwrap_or("null n")
        .split_once(" ")
        .unwrap_or_default()
        .0; // TODO

    let formatted_pkgname = format_pkgbuild(pkgname, &content, src_dir_str);

    let pkg_fn_name;
    let fmt_name = format!("package_{formatted_pkgname}");

    if let Some(_pkg) = result.functions.get("package") {
        println!("=> \x1b[32;1mSUC:\x1b[0m Found package()!");
        pkg_fn_name = "package";

        download_pkgbuild(pkg_fn_name, url, &content, src_dir_str);
    } else {
        println!(
            "=> \x1b[33;1mTRY:\x1b[0m Trying package_{}()",
            formatted_pkgname
        );
        pkg_fn_name = &fmt_name;

        download_pkgbuild(pkg_fn_name, url, &content, src_dir_str);
    }
}

fn format_archive(result: &ParseResult) -> String {
    let archive = result
        .variables
        .get("_archive")
        .map(|s| s.as_str())
        .unwrap_or("null");
    let pkgname: &str = result
        .variables
        .get("pkgname")
        .map(|s| s.as_str())
        .unwrap_or("fycker n")
        .split_once(" ")
        .unwrap_or_default()
        .0; // TODO

    let _pkgname = result
        .variables
        .get("_pkgname")
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

    let formatted = archive
        .replace("$pkgver", pkgver)
        .replace("$pkgname", pkgname)
        .replace("$_pkgname", _pkgname)
        .replace("(", "")
        .replace(")", ""); //TODO: replace with sed;
    formatted
}

fn format_pkgbuild(input: &str, content: &str, src_dir_str: &str) -> String {
    let result: ParseResult = parse(&content);

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
    let pkg_url = result
        .variables
        .get("url")
        .map(|s| s.as_str())
        .unwrap_or("https://popbugs.orgy");

    let formatted = input
        .replace("$srcdir/", src_dir_str)
        .replace("$pkgver", pkgver)
        .replace("$pkgname", pkgname)
        .replace("$_pkgname", _pkgname)
        .replace("$_archive", &format_archive(&result))
        .replace("$pkgbase", pkgbase)
        .replace("$_name", _name)
        .replace("$url", pkg_url)
        .replace("${folder}", "linux-unpacked")
        .replace("${pkgver}", pkgver)
        .replace("${pkgname}", pkgname)
        .replace("{,.sig}", "")
        .replace("git+", "")
        .replace("(", ""); // TODO: could just use sed btw
    // .replace(")", "");

    formatted
}
