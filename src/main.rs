use anyhow::Result;
use regex::Regex;

const DOCS_URL: &str = "http://docs.peachcloud.org/software";
const GITHUB_URL: &str = "https://github.com/peachcloud";
const MANIFEST_URL: &str = "https://raw.githubusercontent.com/peachcloud";

#[derive(Debug)]
struct Service {
    name: String,
    docs_url: String,
    manifest_url: String,
    repo_url: String,
    readme_version: Option<String>,
    docs_version: Option<String>,
    manifest_version: Option<String>,
    //verified: bool,
}

impl Service {
    fn new(name: String) -> Self {
        let docs_url = format!("{}/{}/{}{}", DOCS_URL, "microservices", name, ".html");
        let manifest_url = format!("{}/{}/main/Cargo.toml", MANIFEST_URL, name);
        let repo_url = format!("{}/{}", GITHUB_URL, name);

        Service {
            name,
            docs_url,
            manifest_url,
            repo_url,
            docs_version: None,
            manifest_version: None,
            readme_version: None,
        }
    }

    fn manifest_version(&mut self, body: String) {
        if let Some(version) = regex_finder(r#"version = "(.*)""#, &body) {
            self.manifest_version = Some(version)
        }
    }

    fn readme_version(&mut self, body: String) {
        if let Some(version) = regex_finder(r"badge/version-(.*)-%3C", &body) {
            self.readme_version = Some(version)
        }
    }

    fn docs_version(&mut self, body: String) {
        if let Some(version) = regex_finder(r"badge/version-(.*)-%3C", &body) {
            self.docs_version = Some(version)
        }
    }
}

fn regex_finder(pattern: &str, text: &str) -> Option<String> {
    let re = Regex::new(pattern).unwrap();
    let caps = re.captures(text);
    match caps {
        Some(caps) => Some(caps[1].to_string()),
        None => None,
    }
}

fn main() -> Result<()> {
    let mut oled = Service::new("peach-oled".to_string());

    let docs_body = reqwest::blocking::get(&oled.docs_url)?.text()?;
    let manifest_body = reqwest::blocking::get(&oled.manifest_url)?.text()?;
    let repo_body = reqwest::blocking::get(&oled.repo_url)?.text()?;

    oled.docs_version(docs_body);
    oled.manifest_version(manifest_body);
    oled.readme_version(repo_body);

    println!("{:#?}", oled);

    Ok(())
}
