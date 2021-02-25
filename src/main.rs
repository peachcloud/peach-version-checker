use anyhow::Result;
use regex::Regex;

const DOCS_URL: &str = "http://docs.peachcloud.org/software";
const GITHUB_URL: &str = "https://github.com/peachcloud";

#[derive(Debug)]
struct Service {
    name: String,
    docs_url: String,
    repo_url: String,
    readme_version: Option<String>,
    docs_version: Option<String>,
    //manifest_version: Option<str>,
    //verified: bool,
}

impl Service {
    fn new(name: String) -> Self {
        let docs_url = format!("{}/{}/{}{}", DOCS_URL, "microservices", name, ".html");
        let repo_url = format!("{}/{}", GITHUB_URL, name);

        Service {
            name,
            docs_url,
            repo_url,
            docs_version: None,
            readme_version: None,
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

    let repo_body = reqwest::blocking::get(&oled.repo_url)?.text()?;
    let docs_body = reqwest::blocking::get(&oled.docs_url)?.text()?;

    oled.readme_version(repo_body);
    oled.docs_version(docs_body);

    println!("{:?}", oled);

    Ok(())
}
