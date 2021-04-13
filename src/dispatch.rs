use crate::html;
use crate::error::CheckError;
use std::path::Path;

// If you want to add e.g. CSS checking you'd add 
// Some(e) if e == "css" => css::check_css_file(&path)
// in the pattern match below. and impl check_css_file in it's own mod
pub fn check_file(path: &Path) -> Result<(), CheckError>  {
    let ext = path.extension();
    match ext {
        Some(e) if e == "html" => html::check_html_file(&path),
        _ => Ok(()),
    }
}
