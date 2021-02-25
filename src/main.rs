use anyhow::Result;
use regex::Regex;
use semver::Version;

const DOCS_URL: &str = "http://docs.peachcloud.org/software";
const GITHUB_URL: &str = "https://raw.githubusercontent.com/peachcloud";

#[derive(Debug)]
struct Service {
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
    fn new(name: String) -> Self {
        let docs_url = format!("{}/{}/{}{}", DOCS_URL, "microservices", name, ".html");
        let manifest_url = format!("{}/{}/main/Cargo.toml", GITHUB_URL, name);
        let readme_url = format!("{}/{}/main/README.md", GITHUB_URL, name);

        Service {
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
        let res = reqwest::get(&self.manifest_url).await?;
        let body = res.text().await?;
        if let Some(version) = regex_finder(r#"version = "(.*)""#, &body) {
            self.manifest_version = Some(Version::parse(&version)?)
        }

        Ok(())
    }

    async fn readme_version(&mut self) -> Result<()> {
        let res = reqwest::get(&self.readme_url).await?;
        let body = res.text().await?;
        if let Some(version) = regex_finder(r"badge/version-(.*)-", &body) {
            self.readme_version = Some(Version::parse(&version)?)
        }

        Ok(())
    }

    async fn docs_version(&mut self) -> Result<()> {
        let res = reqwest::get(&self.docs_url).await?;
        let body = res.text().await?;
        if let Some(version) = regex_finder(r"badge/version-(.*)-%3C", &body) {
            self.docs_version = Some(Version::parse(&version)?)
        }

        Ok(())
    }

    async fn check(&mut self) -> Result<()> {
        self.docs_version().await?;
        self.manifest_version().await?;
        self.readme_version().await?;

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
            true => println!("PASS"),
            false => println!("FAIL"),
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

#[tokio::main]
async fn main() -> Result<()> {
    let names = vec!["peach-buttons", "peach-oled", "peach-network"];

    for name in names {
        let mut service = Service::new(name.to_string());
        service.check().await?;
        service.report();
    }

    Ok(())
}
