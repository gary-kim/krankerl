extern crate docopt;
extern crate futures;
extern crate futures_cpupool;
extern crate krankerl;
extern crate nextcloud_appstore;
#[macro_use]
extern crate serde_derive;
extern crate tokio_core;

use docopt::Docopt;
use futures::{future, Future};
use krankerl::*;
use krankerl::packaging::package_app;
use tokio_core::reactor::Core;

const USAGE: &'static str = "
Krankerl. A CLI tool for the Nextcloud app store.

Usage:
  krankerl list apps <version>
  krankerl list categories
  krankerl login [--appstore | --github] <token>
  krankerl package
  krankerl publish (--nightly) <url>
  krankerl sign --package
  krankerl --version

Options:
  -h --help     Show this screen.
  --version     Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_token: Option<String>,
    arg_url: Option<String>,
    arg_version: Option<String>,
    cmd_apps: bool,
    cmd_categories: bool,
    cmd_list: bool,
    cmd_login: bool,
    cmd_package: bool,
    cmd_publish: bool,
    cmd_sign: bool,
    flag_appstore: bool,
    flag_github: bool,
    flag_nightly: bool,
    flag_package: bool,
    flag_version: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let mut core = Core::new().unwrap();
    let mut pool_builder = futures_cpupool::Builder::new();
    pool_builder.pool_size(2);

    if args.cmd_list && args.cmd_apps {
        let version = &args.arg_version.unwrap();

        let work = get_apps_and_releases(&core.handle(), &version.to_owned()).map(|apps| {
            println!("found {} apps for {}:", apps.len(), version);
            for app in apps {
                if app.isFeatured {
                    println!("- {} (featured)", app.id);
                } else {
                    println!("- {}", app.id);
                }
            }
        });

        core.run(work).unwrap();
    } else if args.cmd_list && args.cmd_categories {
        let work = get_categories(&core.handle()).map(|cats| {
            println!("found {} categories:", cats.len());
            for cat in cats {
                println!("- {}", cat.id)
            }
        });

        core.run(work).unwrap();
    } else if args.cmd_login {
        if args.flag_appstore {
            let token = args.arg_token.unwrap();
            config::set_appstore_token(&token).expect("could not save appstore token");
            println!("App store token saved.");
        }
    } else if args.cmd_package {
        package_app().expect("could not package app");
    } else if args.cmd_publish {
        let url = args.arg_url.unwrap();
        let is_nightly = args.flag_nightly;

        let packaging = future::lazy(|| package_app());
        let signing = future::lazy(|| sign_package());
        let handle = core.handle();

        let work = packaging.join(signing.and_then(|signature| {
            let config = config::get_config().expect("could not load config");
            assert!(config.appstore_token.is_some());
            let api_token = config.appstore_token.unwrap();

            publish_app(&handle, &url, is_nightly, &signature, &api_token)
        }));

        core.run(work).unwrap_or_else(|e| {
            println!("an error occured: {:?}", e);
            ((), ())
        });
    } else if args.cmd_sign && args.flag_package {
        let pool = pool_builder.create();
        let work = pool.spawn_fn(|| match sign_package() {
            Ok(signature) => return future::ok(signature),
            Err(err) => return future::err(err),
        }).and_then(|signature| {
                println!("Package signature: {}", signature);
                futures::future::ok(())
            });

        core.run(work).unwrap_or_else(|e| {
            println!("an error occured: {}", e);
        });
    } else if args.flag_version {
        println!("v0.2.0");
    }
}
