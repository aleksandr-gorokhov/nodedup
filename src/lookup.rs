use std::{
    env,
    path::{Component, Path},
};

use walkdir::WalkDir;

pub fn get_package_json_files(dir_path: &str) -> Vec<String> {
    match env::current_dir() {
        Ok(path) => println!("Call directory is: {}", path.display()),
        Err(e) => println!("Error getting call directory: {}", e),
    }
    let absolute_path = Path::new(dir_path).canonicalize().unwrap_or_else(|_| {
        panic!("Failed to resolve the path: {}", dir_path);
    });
    println!("Scanning directory: {}", absolute_path.display());
    WalkDir::new(dir_path)
        .into_iter()
        .filter_entry(|e| !is_node_modules_path(e.path()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| {
            let path = e.path();
            if path.file_name()? == "package.json" {
                path.to_str().map(String::from)
            } else {
                None
            }
        })
        .collect()
}

fn is_node_modules_path(path: &Path) -> bool {
    path.components()
        .any(|c| matches!(c, Component::Normal(os_str) if os_str == "node_modules"))
}

pub fn get_ignore_file(dir_path: &str) -> Option<String> {
    let absolute_path = Path::new(dir_path).canonicalize().unwrap_or_else(|_| {
        panic!("Failed to resolve the path: {}", dir_path);
    });
    let ignore_file_path = absolute_path.join(".ndignore");

    if ignore_file_path.exists() {
        ignore_file_path.to_str().map(String::from)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_return_list_of_package_json_files() {
        let files = get_package_json_files("./src/data/");
        assert_eq!(files.len(), 1);
    }

    #[should_panic]
    #[test]
    fn it_should_panic_for_empty_path() {
        let files = get_package_json_files("");
    }

    #[should_panic]
    #[test]
    fn it_should_panic() {
        let files = get_package_json_files("./.../..");
    }

    #[test]
    fn it_should_return_true_for_node_modules() {
        let path = Path::new("some/path/node_modules");
        assert!(is_node_modules_path(path));
    }

    #[test]
    fn it_should_return_false_for_not_node_modules() {
        let path = Path::new("some/path/no_node_modules");
        assert!(!is_node_modules_path(path));
    }

    #[test]
    fn it_should_find_ignore_file() {
        let file = get_ignore_file("./src/data/");
        assert!(file.is_some(), "Expected Some, got {:?}", file);
    }

    #[test]
    fn it_should_return_empty_string_if_no_file() {
        let file = get_ignore_file("./src");
        assert!(file.is_none());
    }
}
