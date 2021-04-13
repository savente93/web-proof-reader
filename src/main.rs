mod dispatch;
mod error;
mod html;

use walkdir::WalkDir;
use rayon::prelude::*;

use crate::dispatch::*;
use crate::error::CheckError;
 
static TEST_SITE: &str = "public/";

fn main() -> Result<(), String> {
    let (_, errors): (Vec<_>, Vec<_>) = WalkDir::new(&TEST_SITE)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>() 
        .into_par_iter()
        .map(|e| check_file(e.path()))
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
        Err("Errors were found".to_string())
    }
}
