use clap::{crate_authors, crate_description, crate_version, App, Arg};

pub fn build_cli() -> App<'static, 'static> {
    App::new("proof-reader")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("root")
                .index(1)
                .short("r")
                .long("root")
                .takes_value(true)
                .default_value("./public")
                .help("Root of the website to check")
                .required(true)
        )        
        .arg(
            Arg::with_name("exclude")
                .short("e") 
                .long("exclude") 
                .takes_value(true)
                .help("Directory to exclude from search")
        )
}
