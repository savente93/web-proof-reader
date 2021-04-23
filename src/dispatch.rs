use crate::error::CheckError;
use crate::html;
use glob::Pattern;
use std::path::Path;

// If you want to add e.g. CSS checking you'd add
// Some(e) if e == "css" => css::check_css_file(&path)
// in the pattern match below. and impl check_css_file in it's own mod
pub fn check_file(path: &Path, exclude_glob_pattern: Option<&Pattern>) -> Result<(), CheckError> {
    if let Some(glob) = exclude_glob_pattern {
        if glob.matches_path(&path) {
            return Ok(());
        }
    };
    let ext = path.extension();
    match ext {
        Some(e) if e == "html" => html::check_html_file(&path),
        _ => Ok(()),
    }
}
