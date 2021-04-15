mod dispatch;
mod error;
mod html;
mod cli;

use walkdir::WalkDir;
use rayon::prelude::*;
use std::path::PathBuf;

use crate::dispatch::*;
use crate::error::CheckError;
use crate::cli::*;
 
use glob::Pattern;

// static TEST_SITE: &str = "public/";


// TODO configure .pre-commit-config.yaml
fn main() -> Result<(), String> {

    let matches = build_cli().get_matches();

    let root_dir = match matches.value_of("root").unwrap() {
        path => PathBuf::from(path)
            .canonicalize()
            .unwrap_or_else(|_| panic!("Cannot find root directory: {}", path)),
    };

    let exclude = matches.value_of("exclude").map_or_else(|| None, |e| Pattern::new(e).ok());


    let (_, errors): (Vec<_>, Vec<_>) = WalkDir::new(&root_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>() 
        .into_par_iter()
        .map(|e| check_file(e.path(), exclude.as_ref()))
        .partition(Result::is_ok);

    // TODO produce prettier output
    for err in &errors {
        match err {
            Ok(_) => (),
            Err(e) => println!("{}", e),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err("Checks failed!".to_string())
    }
    //TODO figure out post commit to upload build? 
}
