use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use semver::Version;

const DOCS_URL: &str = "http://docs.peachcloud.org/software";
const GITHUB_URL: &str = "https://raw.githubusercontent.com/peachcloud";

#[derive(Debug)]
struct Service {
    client: Client,
    name: String,
    consistent: bool,
    docs_url: String,
    docs_version: Option<Version>,
    manifest_url: String,
    manifest_version: Option<Version>,
    readme_url: String,
    readme_version: Option<Version>,
}

impl Service {
    fn new(client: Client, name: String) -> Self {
        let docs_url = format!("{}/{}/{}{}", DOCS_URL, "microservices", name, ".html");
        let manifest_url = format!("{}/{}/main/Cargo.toml", GITHUB_URL, name);
        let readme_url = format!("{}/{}/main/README.md", GITHUB_URL, name);

        Service {
            client,
            name,
            consistent: false,
            docs_url,
            docs_version: None,
            manifest_url,
            manifest_version: None,
            readme_url,
            readme_version: None,
        }
    }

    async fn manifest_version(&mut self) -> Result<()> {
        // perform GET request
        let res = self.client.get(&self.manifest_url).send().await?;
        // get the full response text
        let body = res.text().await?;
        // pattern match on the text to locate the version number
        if let Some(version) = regex_finder(r#"version = "(.*)""#, &body) {
            // parse the version number into Version and return
            self.manifest_version = Some(Version::parse(&version)?)
        }

        Ok(())
    }

    async fn readme_version(&mut self) -> Result<()> {
        let res = self.client.get(&self.readme_url).send().await?;
        let body = res.text().await?;
        if let Some(version) = regex_finder(r"badge/version-(.*)-", &body) {
            self.readme_version = Some(Version::parse(&version)?)
        }

        Ok(())
    }

    async fn docs_version(&mut self) -> Result<()> {
        let res = self.client.get(&self.docs_url).send().await?;
        let body = res.text().await?;
        if let Some(version) = regex_finder(r"badge/version-(.*)-%3C", &body) {
            self.docs_version = Some(Version::parse(&version)?)
        }

        Ok(())
    }

    async fn check(&mut self) -> Result<()> {
        // retrieve the version numbers for docs, manifest and readme
        self.docs_version().await?;
        self.manifest_version().await?;
        self.readme_version().await?;

        // check for consistency between version numbers
        if self.docs_version == self.manifest_version
            && self.manifest_version == self.readme_version
        {
            self.consistent = true
        } else {
            self.consistent = false
        }

        Ok(())
    }

    fn report(self) {
        let mut terminal = term::stdout().unwrap();
        println!("[ {} ]", self.name);
        match self.docs_version {
            Some(version) => println!("Dev-docs: {}", version),
            None => println!("Dev-docs: No version number found"),
        }
        match self.manifest_version {
            Some(version) => println!("Manifest: {}", version),
            None => println!("Manifest: No version number found"),
        }
        match self.readme_version {
            Some(version) => println!("Readme  : {}", version),
            None => println!("Readme  : No version number found"),
        }
        match self.consistent {
            true => {
                terminal.fg(term::color::GREEN).unwrap();
                terminal.attr(term::Attr::Bold).unwrap();
                println!("PASS");
            }
            false => {
                terminal.fg(term::color::RED).unwrap();
                terminal.attr(term::Attr::Bold).unwrap();
                println!("FAIL");
            }
        }
        // reset the terminal defaults
        terminal.reset().unwrap();
    }
}

// simply helper function for regex pattern matching
fn regex_finder(pattern: &str, text: &str) -> Option<String> {
    let re = Regex::new(pattern).unwrap();
    let caps = re.captures(text);
    match caps {
        Some(caps) => Some(caps[1].to_string()),
        None => None,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // create a new reqwest client (shared connection pool)
    let client = Client::new();

    let microservices = vec![
        "peach-buttons",
        "peach-oled",
        "peach-network",
        "peach-menu",
        "peach-monitor",
        "peach-stats",
    ];

    for microservice in microservices {
        let mut service = Service::new(client.clone(), microservice.to_string());
        service.check().await?;
        service.report();
    }

    let utilities = vec!["peach-probe", "peach-version-checker"];

    for utility in utilities {
        let mut service = Service::new(client.clone(), utility.to_string());
        // utility docs url pattern differs from microservices
        service.docs_url = format!("{}/utilities/{}.html", DOCS_URL, utility.to_string());
        service.check().await?;
        service.report();
    }

    let mut web_interface = Service::new(client.clone(), "peach-web".to_string());
    // peach-web docs url pattern differs from microservices
    web_interface.docs_url = format!("{}/{}", DOCS_URL, "web_interface.html");
    web_interface.check().await?;
    web_interface.report();

    Ok(())
}
