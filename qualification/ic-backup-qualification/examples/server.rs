//! Explicit preparation/admission of the locked Testkit server; never starts it.

use ic_testkit::server::{PreparedServer, host_asset, prepare};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match args.as_slice() {
        [action] if action == "asset" => println!("{}", host_asset()?.url),
        [action, directory] if action == "check" => {
            let prepared = PreparedServer::check(&PathBuf::from(directory))?;
            println!("{}", prepared.path().display());
        }
        [action, archive, directory, gzip] if action == "prepare" => {
            let prepared = prepare(
                &PathBuf::from(archive),
                &PathBuf::from(directory),
                &PathBuf::from(gzip),
            )?;
            println!("{}", prepared.path().display());
        }
        _ => {
            return Err(
                "usage: server asset | check DIRECTORY | prepare ARCHIVE DIRECTORY ABSOLUTE_GZIP"
                    .into(),
            );
        }
    }

    Ok(())
}
