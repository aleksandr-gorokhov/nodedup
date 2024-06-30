use std::{collections::HashMap, fs, path::Path};

use serde_json::Value;

#[derive(Debug, PartialEq, Clone)]
pub struct PackageValue {
    pub name: String,
    pub version: String,
    pub path: String,
}

impl PackageValue {
    pub fn new(name: &str, version: &str, path: &str) -> Self {
        PackageValue {
            name: name.to_string(),
            version: version.to_string(),
            path: path.to_string(),
        }
    }
}

fn parse_file(path: &Path) -> std::io::Result<Value> {
    let res = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&res)?;

    Ok(value)
}

fn build_hash_map(value: Value, path: &str, map: &mut HashMap<String, Vec<PackageValue>>) {
    let deps = value.get("dependencies");
    let dev_deps = value.get("devDependencies");
    traverse_deps(deps, map, path);
    traverse_deps(dev_deps, map, path);
}

fn traverse_deps(deps: Option<&Value>, map: &mut HashMap<String, Vec<PackageValue>>, path: &str) {
    deps.and_then(|d| d.as_object())
        .into_iter()
        .flat_map(|deps| deps.iter())
        .filter_map(|(key, value)| Some((key, value.as_str()?)))
        .for_each(|(key, value_str)| process_dependency(key, value_str, map, path));
}

fn process_dependency(
    key: &str,
    value_str: &str,
    map: &mut HashMap<String, Vec<PackageValue>>,
    path: &str,
) {
    let entry = map.entry(key.to_string()).or_default();
    let version = clean_version(value_str);
    let package_value = PackageValue::new(key, &version, path);

    if entry.iter().any(|v| v.version == version) {
        return;
    }

    let should_unshift = is_new_version_higher(&version, entry);

    if should_unshift {
        entry.insert(0, package_value);
    } else {
        entry.push(package_value);
    }
}

fn clean_version(version_str: &str) -> String {
    version_str
        .chars()
        .filter(|c| *c == '.' || c.is_ascii_digit())
        .collect()
}

fn is_new_version_higher(version: &str, entry: &[PackageValue]) -> bool {
    if entry.is_empty() {
        return false;
    }

    let (major, minor, patch) = get_versions(&entry[0].version);
    let (major_new, minor_new, patch_new) = get_versions(version);

    major_new > major
        || (major_new == major && minor_new > minor)
        || (major_new == major && minor_new == minor && patch_new > patch)
}

fn get_versions(version: &str) -> (u32, u32, u32) {
    let mut parts = version.split('.');
    let major = parts.next().unwrap_or("0").parse().unwrap_or(0);
    let minor = parts.next().unwrap_or("0").parse().unwrap_or(0);
    let patch = parts.next().unwrap_or("0").parse().unwrap_or(0);

    (major, minor, patch)
}

pub fn find_duplicate_dependencies(
    paths: Vec<String>,
    ignore_path: &str,
) -> HashMap<String, Vec<PackageValue>> {
    let ignore_file = read_ignores(ignore_path);
    let ignores = parse_ignores(&ignore_file.unwrap_or_default());
    let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
    for path in paths {
        let path_buf = Path::new(&path);
        let value = parse_file(path_buf).unwrap();
        build_hash_map(value, &path, &mut hash_map);
    }
    keep_bad_values(&mut hash_map, ignores);

    hash_map
}

fn keep_bad_values(hash_map: &mut HashMap<String, Vec<PackageValue>>, ignores: Vec<String>) {
    let keys_to_remove: Vec<String> = hash_map
        .iter()
        .filter_map(|(key, values)| {
            let ignored = ignores.iter().any(|i| i == key);
            if values.len() > 1 && !ignored {
                None
            } else {
                Some(key.clone())
            }
        })
        .collect();

    for key in keys_to_remove {
        hash_map.remove(&key);
    }
}

fn read_ignores(path: &str) -> std::io::Result<String> {
    let file = fs::read_to_string(path)?;

    Ok(file)
}

fn parse_ignores(ignores: &str) -> Vec<String> {
    ignores.lines().map(|s| s.trim().to_string()).collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn it_should_read_package_json() {
        let path = PathBuf::from("./src/data/package.json");

        let actual = parse_file(&path).unwrap();

        let expected = r#"{
          "dependencies": {
            "mongoose": "^1.0.0"
          }
        }"#;
        let expected: Value = serde_json::from_str(expected).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_read_ignore_file() {
        let actual = read_ignores("./src/data/.ndignore").unwrap();

        let expected = "testignore\ntestignore2\ntestignore3";

        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_build_hash_map() {
        let json = r#"{
          "dependencies": {
            "mongoose": "^1.0.0"
          }
        }"#;
        let parsed: Value = serde_json::from_str(json).unwrap();
        let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
        build_hash_map(parsed, "", &mut hash_map);

        let mut result_hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
        result_hash_map.insert(
            "mongoose".to_string(),
            vec![PackageValue::new("mongoose", "1.0.0", "")],
        );
        assert_eq!(hash_map, result_hash_map);
    }

    #[test]
    fn it_should_add_two_values() {
        let json1 = r#"{
          "dependencies": {
            "mongoose": "^1.0.0"
          }
        }"#;
        let json2 = r#"{
          "dependencies": {
            "mongoose": "2.0.0"
          }
        }"#;
        let parsed1: Value = serde_json::from_str(json1).unwrap();
        let parsed2: Value = serde_json::from_str(json2).unwrap();
        let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
        build_hash_map(parsed1, "", &mut hash_map);
        build_hash_map(parsed2, "", &mut hash_map);

        let mut result_hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
        result_hash_map.insert(
            "mongoose".to_string(),
            vec![
                PackageValue::new("mongoose", "2.0.0", ""),
                PackageValue::new("mongoose", "1.0.0", ""),
            ],
        );

        assert_eq!(hash_map, result_hash_map);
    }

    #[test]
    fn it_should_call_all_together() {
        let path = "./src/data/package.json".to_string();
        let result = find_duplicate_dependencies(vec![path], "");

        assert_eq!(result, HashMap::new());
    }

    #[test]
    fn it_should_also_parse_dev_dependencies() {
        let json1 = r#"{
          "dependencies": {
            "mongoose": "^1.0.0"
          }
        }"#;
        let json2 = r#"{
          "devDependencies": {
            "mongoose": "2.0.0"
          }
        }"#;
        let parsed1: Value = serde_json::from_str(json1).unwrap();
        let parsed2: Value = serde_json::from_str(json2).unwrap();
        let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
        build_hash_map(parsed1, "", &mut hash_map);
        build_hash_map(parsed2, "", &mut hash_map);

        let mut result_hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
        result_hash_map.insert(
            "mongoose".to_string(),
            vec![
                PackageValue::new("mongoose", "2.0.0", ""),
                PackageValue::new("mongoose", "1.0.0", ""),
            ],
        );

        assert_eq!(hash_map, result_hash_map);
    }

    #[test]
    fn it_should_return_struct_with_path() {
        let json = r#"{
          "dependencies": {
            "mongoose": "^1.0.0"
          }
        }"#;
        let parsed: Value = serde_json::from_str(json).unwrap();
        let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();

        let path = "./src/data/package.json";
        build_hash_map(parsed, path, &mut hash_map);

        let mut result_hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
        result_hash_map.insert(
            "mongoose".to_string(),
            vec![PackageValue::new("mongoose", "1.0.0", path)],
        );

        assert_eq!(hash_map, result_hash_map);
    }

    #[test]
    fn it_should_sort_correctly() {
        let json1 = r#"{
          "dependencies": {
            "mongoose": "^1.0.0"
          }
        }"#;
        let json2 = r#"{
          "devDependencies": {
            "mongoose": "2.0.0"
          }
        }"#;

        let json3 = r#"{
          "devDependencies": {
            "mongoose": "2.1.0"
          }
        }"#;
        let json4 = r#"{
          "devDependencies": {
            "mongoose": "2.1.1"
          }
        }"#;
        let json5 = r#"{
          "devDependencies": {
            "mongoose": "2.0.1"
          }
        }"#;
        let parsed1: Value = serde_json::from_str(json1).unwrap();
        let parsed2: Value = serde_json::from_str(json2).unwrap();
        let parsed3: Value = serde_json::from_str(json3).unwrap();
        let parsed4: Value = serde_json::from_str(json4).unwrap();
        let parsed5: Value = serde_json::from_str(json5).unwrap();
        let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();

        build_hash_map(parsed1, "", &mut hash_map);
        build_hash_map(parsed2, "", &mut hash_map);
        build_hash_map(parsed3, "", &mut hash_map);
        build_hash_map(parsed4, "", &mut hash_map);
        build_hash_map(parsed5, "", &mut hash_map);

        let mut result_hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
        result_hash_map.insert(
            "mongoose".to_string(),
            vec![
                PackageValue::new("mongoose", "2.1.1", ""),
                PackageValue::new("mongoose", "2.1.0", ""),
                PackageValue::new("mongoose", "2.0.0", ""),
                PackageValue::new("mongoose", "1.0.0", ""),
                PackageValue::new("mongoose", "2.0.1", ""),
            ],
        );

        assert_eq!(hash_map, result_hash_map);
    }

    #[test]
    fn it_should_skip_same_versions() {
        let json1 = r#"{
          "dependencies": {
            "mongoose": "^1.0.0"
          }
        }"#;
        let json2 = r#"{
          "devDependencies": {
            "mongoose": "1.0.0"
          }
        }"#;

        let parsed1: Value = serde_json::from_str(json1).unwrap();
        let parsed2: Value = serde_json::from_str(json2).unwrap();
        let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();

        build_hash_map(parsed1, "", &mut hash_map);
        build_hash_map(parsed2, "", &mut hash_map);

        let mut result_hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
        result_hash_map.insert(
            "mongoose".to_string(),
            vec![PackageValue::new("mongoose", "1.0.0", "")],
        );

        assert_eq!(hash_map, result_hash_map);
    }

    #[test]
    fn it_should_correctly_sort_versions() {
        let json1 = r#"{
          "dependencies": {
            "mongoose": "^1.3.0"
          }
        }"#;
        let json2 = r#"{
          "devDependencies": {
            "mongoose": "1.10.0"
          }
        }"#;

        let parsed1: Value = serde_json::from_str(json1).unwrap();
        let parsed2: Value = serde_json::from_str(json2).unwrap();
        let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();

        build_hash_map(parsed1, "", &mut hash_map);
        build_hash_map(parsed2, "", &mut hash_map);

        let mut result_hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
        result_hash_map.insert(
            "mongoose".to_string(),
            vec![
                PackageValue::new("mongoose", "1.10.0", ""),
                PackageValue::new("mongoose", "1.3.0", ""),
            ],
        );

        assert_eq!(hash_map, result_hash_map);
    }

    #[test]
    fn it_should_parse_ignore() {
        let parsed = parse_ignores("mongoose\nexpress\n");
        assert_eq!(parsed, vec!["mongoose", "express"]);
    }

    #[test]
    fn it_should_return_empty_ignore() {
        let parsed = parse_ignores("");
        assert!(parsed.is_empty());
    }

    mod keep_bad_values {
        use super::*;

        #[test]
        fn it_should_remove_valid_deps() {
            let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
            hash_map.insert(
                "mongoose".to_string(),
                vec![
                    PackageValue::new("mongoose", "2.0.0", "path/to/mongoose"),
                    PackageValue::new("mongoose", "1.0.0", "path/to/mongoose"),
                ],
            );
            hash_map.insert(
                "test".to_string(),
                vec![PackageValue::new("test", "2.0.0", "path/to/mongoose")],
            );

            let mut result_hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
            result_hash_map.insert(
                "mongoose".to_string(),
                vec![
                    PackageValue::new("mongoose", "2.0.0", "path/to/mongoose"),
                    PackageValue::new("mongoose", "1.0.0", "path/to/mongoose"),
                ],
            );

            keep_bad_values(&mut hash_map, vec![]);

            assert_eq!(hash_map, result_hash_map);
        }

        #[test]
        fn it_should_ignore() {
            let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
            hash_map.insert(
                "mongoose".to_string(),
                vec![
                    PackageValue::new("mongoose", "2.0.0", "path/to/mongoose"),
                    PackageValue::new("mongoose", "1.0.0", "path/to/mongoose"),
                ],
            );

            keep_bad_values(&mut hash_map, vec!["mongoose".to_string()]);

            assert_eq!(hash_map, HashMap::new());
        }

        #[test]
        fn it_should_ignore_more_than_one_value() {
            let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
            hash_map.insert(
                "mongoose".to_string(),
                vec![
                    PackageValue::new("mongoose", "2.0.0", "path/to/mongoose"),
                    PackageValue::new("mongoose1", "2.0.0", "path/to/mongoose"),
                    PackageValue::new("mongoose1", "1.0.0", "path/to/mongoose"),
                    PackageValue::new("mongoose", "1.0.0", "path/to/mongoose"),
                ],
            );

            keep_bad_values(
                &mut hash_map,
                vec!["mongoose".to_string(), "mongoose1".to_string()],
            );

            assert_eq!(hash_map, HashMap::new());
        }

        #[test]
        fn it_should_return_empty_vec_for_good_values() {
            let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
            hash_map.insert(
                "mongoose".to_string(),
                vec![PackageValue::new("mongoose", "1.0.0", "")],
            );

            keep_bad_values(&mut hash_map, vec![]);

            assert_eq!(hash_map, HashMap::new());
        }
    }
}
