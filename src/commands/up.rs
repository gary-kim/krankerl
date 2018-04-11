use std::path::PathBuf;

use failure::Error;
use composer::Composer;
use npm_scripts::NpmScripts;

fn npm_up(app_path: &PathBuf) -> Result<(), Error> {
    let npm_script = "build".to_owned();
    let scripts = NpmScripts::new(app_path);

    if !scripts.is_available() {
        println!("no package.json available, skipping npm installation");
        return Ok(());
    }
    scripts.install()?;

    let has_npm_build_task = scripts.has_script(&npm_script)?;
    if has_npm_build_task {
        scripts.run_script(&npm_script)?;
        println!("Ran npm build script.");
    } else {
        bail!("no package.json or npm build task found");
    }
    Ok(())
}

fn composer_up(app_path: &PathBuf) -> Result<(), Error> {
    let composer = Composer::new(app_path);
    if composer.is_available() {
        composer.install()?;
        println!("Installed composer packages.");
    } else {
        println!("no composer.json found, skipping composer installation");
    }
    Ok(())
}

pub fn up(app_path: &PathBuf) -> Result<(), Error> {
    npm_up(app_path)?;
    composer_up(app_path)?;
    Ok(())
}
