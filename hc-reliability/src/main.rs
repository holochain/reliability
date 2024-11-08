use regex::Regex;

#[derive(Debug, serde::Deserialize)]
struct Asset {
    pub browser_download_url: String,
    pub name: String,
}

#[derive(serde::Deserialize)]
struct Release {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

impl std::fmt::Debug for Release {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rel_name = self.rel_name();
        let rel_ver = self.rel_ver();
        f.debug_struct("Release")
            .field("rel_name", &rel_name)
            .field("rel_ver", &rel_ver)
            .field("tag_name", &self.tag_name)
            .field("assets", &self.assets)
            .finish()
    }
}

impl Release {
    pub fn rel_name(&self) -> String {
        let re = Regex::new("(hc-reliability-ui|hc-reliability)").unwrap();
        re.captures(&self.tag_name)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .to_string()
    }

    pub fn rel_ver(&self) -> String {
        let re = Regex::new("v(\\d\\.\\d\\.\\d)").unwrap();
        re.captures(&self.tag_name)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .to_string()
    }
}

fn main() {
    let releases = ureq::get(
        "https://api.github.com/repos/holochain/reliability/releases",
    )
    .set("Accept", "application/vnd.github+json")
    .set("X-GitHub-Api-Version", "2022-11-28")
    .call()
    .expect("Failed to call github releases api")
    .into_string()
    .expect("Failed to parse github releases api response");
    let releases: Vec<Release> = serde_json::from_str(&releases)
        .expect("Failed to parse github releases api response json");
    println!("{releases:#?}");
    let release = releases
        .into_iter()
        .filter(|r| r.rel_name() == "hc-reliability-ui")
        .next()
        .unwrap();
    let asset = release
        .assets
        .into_iter()
        .filter(|a| a.name == "hc-reliability-ui")
        .next()
        .unwrap();
    println!("{asset:#?}");

    let mut asset = ureq::get(&asset.browser_download_url)
        .call()
        .expect("Failed to download hc-reliability-ui")
        .into_reader();

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open("hc-reliability-ui")
        .unwrap();

    std::io::copy(&mut asset, &mut file).unwrap();

    #[cfg(unix)]
    {
        let meta = file.metadata().unwrap();
        let mut perm = meta.permissions();
        use std::os::unix::fs::PermissionsExt;
        perm.set_mode(0o755);
        file.set_permissions(perm).unwrap();
    }

    drop(asset);
    drop(file);

    assert!(std::process::Command::new("./hc-reliability-ui").status().unwrap().success());
}
