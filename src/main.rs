use clap::Parser;

mod lookup;
mod parser;

/// Find duplicate JS dependencies
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Folder to scan
    #[arg(short, long)]
    folder: String,

    /// Exit with zero code when duplicates are found
    #[arg(short, long, default_value_t = false)]
    silent: bool,
}

fn main() {
    let args = Args::parse();

    let folder = args.folder;
    let ignore = lookup::get_ignore_file(&folder);
    let files = lookup::get_package_json_files(&folder);
    let duplicates = parser::find_duplicate_dependencies(files, &ignore.unwrap_or_default());
    println!("Duplicates: {:#?}", duplicates);
    println!("Total duplicates: {}", duplicates.len());

    if args.silent {
        std::process::exit(0);
    }
    std::process::exit(duplicates.len() as i32);
}
