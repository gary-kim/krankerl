use std::path::PathBuf;
use std::thread;

use failure::Error;
use indicatif::{MultiProgress, ProgressBar};
use composer::Composer;
use npm_scripts::NpmScripts;

use console::default_spinner;

fn find_npm_scripts(app_path: &PathBuf) -> Option<NpmScripts> {
    let in_root = NpmScripts::new(app_path);
    let in_js = NpmScripts::new(app_path.join("js"));

    if in_root.is_available() {
        Some(in_root)
    } else if in_js.is_available() {
        Some(in_js)
    } else {
        None
    }
}

fn npm_up(app_path: &PathBuf, pb: ProgressBar) -> Result<(), Error> {
    pb.enable_steady_tick(200);

    let npm_script = "build".to_owned();

    match find_npm_scripts(app_path) {
        Some(scripts) => {
            pb.set_message(&format!("Installing npm packages..."));
            scripts.install()?;
            let has_npm_build_task = scripts.has_script(&npm_script)?;
            if has_npm_build_task {
                pb.set_message(&format!("Running npm build script..."));
                scripts.run_script(&npm_script)?;
                pb.finish_with_message(&format!("Installed npm packages and ran build script."));
            } else {
                pb.finish_with_message(&format!("Installed npm packages."));
            }
            Ok(())
        }
        None => {
            pb.finish_with_message(&format!("No npm config found."));
            Ok(())
        }
    }
}

fn composer_up(app_path: &PathBuf, pb: ProgressBar) -> Result<(), Error> {
    pb.enable_steady_tick(200);

    let composer = Composer::new(app_path);
    if composer.is_available() {
        pb.set_message(&format!("Installing composer packages..."));
        composer.install()?;
        pb.finish_with_message(&format!("Installed composer packages."));
    } else {
        pb.finish_with_message(&format!("No composer config found."));
    }
    Ok(())
}

pub fn up(app_path: &PathBuf) -> Result<(), Error> {
    let m = MultiProgress::new();

    let pb = m.add(default_spinner());
    let p1 = app_path.to_owned();
    let t1 = thread::spawn(move || npm_up(&p1, pb));
    let pb = m.add(default_spinner());
    let p2 = app_path.to_owned();
    let t2 = thread::spawn(move || composer_up(&p2, pb));

    m.join()?;
    t1.join().unwrap()?;
    t2.join().unwrap()?;

    Ok(())
}
