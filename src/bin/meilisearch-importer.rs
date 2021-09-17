use clap::{App, Arg};
use color_eyre::Report;
use xapiary::util::glob_files;
use xapiary::xq_document::parse_file;

fn setup() -> Result<(), Report> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install()?;

    Ok(())
}

fn main() -> Result<(), Report> {
    setup()?;

    let cli = App::new("meilisearch-importer")
        .version("1.0")
        .author("Steve <steve@little-fluffy.cloud>")
        .about("Read my vimdiary markdown files and import them into local Meilisearch")
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("globpath") // And their own arguments
                .help("the files to add")
                .required(true),
        )
        .get_matches();

    let verbosity = cli.occurrences_of("v");

    let client = reqwest::blocking::Client::new();

    // Read the markdown files and post them to local Meilisearch
    for entry in glob_files(
        cli.value_of("globpath").unwrap(),
        cli.occurrences_of("v") as i8,
    )
    .expect("Failed to read glob pattern")
    {
        match entry {
            // TODO convert this to iterator style using map/filter
            Ok(path) => {
                if let Ok(mut xqdoc) = parse_file(&path) {
                    let out = xqdoc.clone();
                    let res = client
                        .post("http://127.0.0.1:7700/indexes/notes/documents")
                        .body(serde_json::to_string(&vec![xqdoc]).unwrap())
                        .send()?;
                    if verbosity > 0 {
                        println!(
                            "✅ {:?} {}",
                            res, serde_json::to_string(&vec![out]).unwrap(),
                        );
                    }
                } else {
                    eprintln!("❌ Failed to load file {}", path.display());
                }
            }

            Err(e) => eprintln!("❌ {:?}", e),
        }
    }

    Ok(())
}
