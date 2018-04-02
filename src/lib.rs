extern crate base64;
extern crate flate2;
#[cfg(test)]
extern crate fs_extra;
extern crate futures;
extern crate futures_cpupool;
extern crate globset;
extern crate hex;

extern crate nextcloud_appinfo;
extern crate nextcloud_appsignature;
extern crate nextcloud_appstore;
extern crate openssl;
extern crate pathdiff;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tar;
#[cfg(test)]
extern crate tempdir;
extern crate tokio_core;
extern crate toml;
extern crate walkdir;
extern crate xdg;

pub mod config;
pub mod error;
pub mod occ;
pub mod packaging;

use std::env;
use std::path::{Path, PathBuf};

use futures::Future;
use nextcloud_appinfo::get_appinfo;
pub use nextcloud_appstore::{get_apps_and_releases, get_categories};
use tokio_core::reactor::Handle;
use occ::Occ;

pub fn enable_app() -> Result<(), error::Error> {
    let app_path = Path::new(".").canonicalize()?;
    let info = get_appinfo(&app_path)?;
    let occ = Occ::new("../../occ");
    occ.enable_app(info.id())
}

pub fn disable_app() -> Result<(), error::Error> {
    let app_path = Path::new(".").canonicalize()?;
    let info = get_appinfo(&app_path)?;
    let occ = Occ::new("../../occ");
    occ.disable_app(info.id())
}

fn get_home_dir() -> Result<PathBuf, error::Error> {
    env::home_dir().ok_or(error::Error::Other("Could not resolve home dir".to_string()))
}

fn get_private_key_path(app_id: &String) -> Result<PathBuf, error::Error> {
    let mut key_path = get_home_dir()?;
    key_path.push(".nextcloud");
    key_path.push("certificates");
    key_path.push(app_id.to_string() + ".key");
    Ok(key_path)
}

fn get_package_path(app_id: &String) -> Result<PathBuf, error::Error> {
    let mut path = PathBuf::from(".").canonicalize()?;
    path.push("build");
    path.push("artifacts");
    path.push(app_id.to_string() + ".tar.gz");
    Ok(path)
}

pub fn sign_package() -> Result<String, error::Error> {
    let app_path = Path::new(".").canonicalize()?;
    let appinfo = get_appinfo(&app_path)?;
    let app_id = appinfo.id();
    let key_path = get_private_key_path(app_id)?;
    let package_path = get_package_path(app_id)?;

    if !package_path.exists() {
        return Err(error::Error::Other("No package found".to_string()));
    }

    let signature = nextcloud_appsignature::sign_package(&key_path, &package_path)?;

    Ok(signature)
}

pub fn publish_app(handle: &Handle,
                   url: &String,
                   is_nightly: bool,
                   signature: &String,
                   api_token: &String)
                   -> Box<futures::Future<Item = (), Error = error::Error>> {
    Box::new(nextcloud_appstore::publish_app(handle, url, is_nightly, signature, api_token)
                 .map_err(|e| error::Error::AppStore(e)))
}
