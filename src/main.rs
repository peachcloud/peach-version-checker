use regex::Regex;

const DOCS_URL: &'static str = "http://docs.peachcloud.org/software";
const GITHUB_URL: &'static str = "https://github.com/peachcloud";

struct Service {
    name: String,
    repo_url: String,
    //repo_version: str,
    //docs_url: str,
    //docs_version: str,
    //manifest_version: Option<str>,
    //verified: bool,
}

impl Service {
    fn new(name: String) -> Self {
        let repo_url = format!("{}/{}", GITHUB_URL, name);

        Service { name, repo_url }
    }

    fn readme_version(self, body: String) -> Option<String> {
        regex_finder(r"badge/version-(.*)-%3C", &body)
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let buttons = Service::new("peach-buttons".to_string());

    let body = reqwest::blocking::get(&buttons.repo_url)?.text()?;

    match buttons.readme_version(body) {
        Some(version) => println!("{}", version),
        None => ()
    }

    Ok(())
}
