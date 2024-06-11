use std::env;

mod lookup;
mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path>", args[0]);
        std::process::exit(1);
    }
    let path = &args[1];
    let files = lookup::get_package_json_files(path);
    let duplicates = parser::find_duplicate_dependencies(files);
    println!("Duplicates: {:#?}", duplicates);
    println!("Total duplicates: {}", duplicates.len());
}
