mod cli;
mod config;
mod date;
mod dispatch;
mod error;
mod html;

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::cli::*;
use crate::config::{build_config, ReaderConfig};
use crate::dispatch::*;
use crate::error::CheckError;

use std::path::Path;

fn main() -> Result<(), String> {
    let matches = build_cli().get_matches();

    let conf: ReaderConfig = matches
        .value_of("config")
        .map_or_else(ReaderConfig::default, |p| {
            build_config(Some(Path::new(p))).unwrap()
        });

    let (_, errors): (Vec<_>, Vec<_>) = WalkDir::new(&conf.root_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|e| check_file(e.path(), &conf))
        .partition(Result::is_ok);

    for err in &errors {
        match err {
            Ok(_) => (),
            Err(e) => println!("{}", e),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("{} Checks failed!", errors.len()))
    }
}
