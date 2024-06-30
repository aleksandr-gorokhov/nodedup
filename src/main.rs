use clap::Parser;

use crate::formatter::DependenciesFormatter;

mod formatter;
mod lookup;
mod parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Folder to scan
    #[arg(short, long)]
    folder: String,

    /// Output format. Possible values: 'default', 'short', 'full'
    #[arg(short, long, default_value = "default")]
    output: String,

    /// Exit with zero code when duplicates are found
    #[arg(short, long)]
    silent: bool,

    /// Color important output
    #[arg(short, long)]
    color: bool,
}

fn main() {
    let args = Args::parse();

    let folder = args.folder;
    let ignore = lookup::get_ignore_file(&folder);
    let files = lookup::get_package_json_files(&folder);
    let duplicates = parser::find_duplicate_dependencies(files, &ignore.unwrap_or_default());
    let errors = duplicates.len() as i32;
    let mut formatter = DependenciesFormatter::new(duplicates);
    formatter.try_set_style(&args.output);
    let result = formatter.format(args.color);
    println!("{}", result);

    if args.silent {
        std::process::exit(0);
    }
    std::process::exit(errors);
}
