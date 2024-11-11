use crate::*;

const APP_INFO: app_dirs2::AppInfo = app_dirs2::AppInfo {
    name: "hc-reliability",
    author: "holochain",
};

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
    let releases = match ureq::get(
        "https://api.github.com/repos/holochain/reliability/releases",
    )
    .set("Accept", "application/vnd.github+json")
    .set("X-GitHub-Api-Version", "2022-11-28")
    .call() {
        Ok(releases) => releases,
        _ => return None,
    };

    let releases = match releases.into_string() {
        Ok(releases) => releases,
        _ => return None,
    };

    let releases: Vec<Release> = match serde_json::from_str(&releases) {
        Ok(releases) => releases,
        _ => return None,
    };

    let release = match releases
        .into_iter()
        .filter(|r| {
            match semver::Version::parse(&r.tag_name) {
                Ok(v) => true, //v > cur,
                _ => false
            }
        })
        .next() {
            Some(release) => release,
            None => return None,
        };

    let want_asset = format!("hc-reliability-{OS}-{ARCH}-v{VERSION}.zip");
    println!("looking for asset {want_asset}");

    let asset = match release
        .assets
        .into_iter()
        .filter(|a| a.name == want_asset)
        .next() {
            Some(asset) => asset,
            None => return None,
        };

    println!("{asset:#?}");

    let mut asset = ureq::get(&asset.browser_download_url)
        .call()
        .expect("Failed to download hc-reliability")
        .into_reader();

    let cache = app_dirs2::get_app_dir(app_dirs2::AppDataType::UserCache, &APP_INFO, "updates").unwrap_or_else(|_| std::env::current_dir().unwrap());

    let _ = std::fs::create_dir_all(&cache);
    let zip = cache.join(want_asset);

    println!("{zip:?}");
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(zip)
        .unwrap();

    std::io::copy(&mut asset, &mut file).unwrap();

    /*
    #[cfg(unix)]
    {
        let meta = file.metadata().unwrap();
        let mut perm = meta.permissions();
        use std::os::unix::fs::PermissionsExt;
        perm.set_mode(0o755);
        file.set_permissions(perm).unwrap();
    }
    */

    drop(asset);

    use std::io::Seek;
    file.rewind().unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();
    archive.extract(cache).unwrap();

    drop(archive);

    //assert!(std::process::Command::new("./hc-reliability").status().unwrap().success());

    None
}
