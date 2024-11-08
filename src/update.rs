#[derive(Debug, serde::Deserialize)]
struct Asset {
    pub browser_download_url: String,
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
struct Release {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

pub fn try_update(cur: semver::Version) -> Option<std::path::PathBuf> {
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

    let release = match releases
        .into_iter()
        .filter(|r| {
            match semver::Version::parse(&r.tag_name) {
                Ok(v) => v > cur,
                _ => false
            }
        })
        .next() {
            Some(release) => release,
            None => return None,
        };

    let asset = match release
        .assets
        .into_iter()
        .filter(|a| a.name == "hc-reliability")
        .next() {
            Some(asset) => asset,
            None => return None,
        };

    println!("{asset:#?}");

    let mut asset = ureq::get(&asset.browser_download_url)
        .call()
        .expect("Failed to download hc-reliability")
        .into_reader();

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open("hc-reliability")
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

    //assert!(std::process::Command::new("./hc-reliability").status().unwrap().success());

    None
}
