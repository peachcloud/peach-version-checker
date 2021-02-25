use anyhow::Result;
use regex::Regex;

const DOCS_URL: &str = "http://docs.peachcloud.org/software";
const GITHUB_URL: &str = "https://raw.githubusercontent.com/peachcloud";

#[derive(Debug)]
struct Service {
    name: String,
    docs_url: String,
    docs_version: Option<String>,
    manifest_url: String,
    manifest_version: Option<String>,
    readme_url: String,
    readme_version: Option<String>,
}

impl Service {
    fn new(name: String) -> Self {
        let docs_url = format!("{}/{}/{}{}", DOCS_URL, "microservices", name, ".html");
        let manifest_url = format!("{}/{}/main/Cargo.toml", GITHUB_URL, name);
        let readme_url = format!("{}/{}/main/README.md", GITHUB_URL, name);

        Service {
            name,
            docs_url,
            docs_version: None,
            manifest_url,
            manifest_version: None,
            readme_url,
            readme_version: None,
        }
    }

    fn manifest_version(&mut self, body: String) {
        if let Some(version) = regex_finder(r#"version = "(.*)""#, &body) {
            self.manifest_version = Some(version)
        }
    }

    fn readme_version(&mut self, body: String) {
        if let Some(version) = regex_finder(r"badge/version-(.*)-", &body) {
            self.readme_version = Some(version)
        }
    }

    fn docs_version(&mut self) -> Result<()> {
        let body = reqwest::blocking::get(&self.docs_url)?.text()?;
        if let Some(version) = regex_finder(r"badge/version-(.*)-%3C", &body) {
            self.docs_version = Some(version)
        }

        Ok(())
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

    let manifest_body = reqwest::blocking::get(&oled.manifest_url)?.text()?;
    let readme_body = reqwest::blocking::get(&oled.readme_url)?.text()?;

    // slightly different approach: make request in method
    oled.docs_version()?;

    // pass request body into method
    oled.manifest_version(manifest_body);
    oled.readme_version(readme_body);

    println!("{:#?}", oled);

    Ok(())
}
