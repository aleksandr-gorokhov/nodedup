use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

#[derive(Debug, PartialEq, Clone)]
struct PackageValue {
    name: String,
    version: String,
    path: String,
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
    if let Some(deps) = deps {
        if let Some(deps) = deps.as_object() {
            for (key, value) in deps {
                if let Some(value_str) = value.as_str() {
                    let entry = map.entry(key.to_string()).or_default();

                    let cleaned_value = value_str
                        .chars()
                        .filter(|c| *c == '.' || c.is_ascii_digit())
                        .collect::<String>();

                    if !entry.iter_mut().any(|v| v.version == cleaned_value) {
                        if entry.is_empty() {
                            entry.push(PackageValue {
                                name: key.to_string(),
                                version: cleaned_value.to_string(),
                                path: path.to_string(),
                            });
                            continue;
                        }
                        let (major, minor, patch) = get_versions(&entry[0].version);
                        let (major_new, minor_new, patch_new) = get_versions(&cleaned_value);

                        let should_insert = major_new > major
                            || (major_new == major && minor_new > minor)
                            || (major_new == major && minor_new == minor && patch_new > patch);

                        if should_insert {
                            entry.insert(
                                0,
                                PackageValue {
                                    name: key.to_string(),
                                    version: cleaned_value.to_string(),
                                    path: path.to_string(),
                                },
                            );
                            continue;
                        }

                        entry.push(PackageValue {
                            name: key.to_string(),
                            version: cleaned_value.to_string(),
                            path: path.to_string(),
                        });
                    }
                }
            }
        }
    }
}

fn get_versions(version: &str) -> (u32, u32, u32) {
    let mut parts = version.split('.');
    let major = parts.next().unwrap_or("0").parse().unwrap_or(0);
    let minor = parts.next().unwrap_or("0").parse().unwrap_or(0);
    let patch = parts.next().unwrap_or("0").parse().unwrap_or(0);

    (major, minor, patch)
}

fn find_bad_values(hash_map: &HashMap<String, Vec<PackageValue>>) -> Vec<String> {
    let mut bad_values = Vec::new();

    for (key, values) in hash_map {
        if values.len() > 1 {
            let result = format!(
                "Package: {}. Unique versions: {}. Highest version: {}. Located: {}.",
                key,
                values.len(),
                values[0].version,
                values[0].path,
            );
            bad_values.push(result);
        }
    }

    bad_values
}

pub fn find_duplicate_dependencies(paths: Vec<String>) -> Vec<String> {
    let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
    for path in paths {
        let path_buf = Path::new(&path);
        let value = parse_file(path_buf).unwrap();
        build_hash_map(value, &path, &mut hash_map);
    }
    let bad_values = find_bad_values(&hash_map);

    bad_values.iter().map(|s| s.to_string()).collect()
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
            vec![PackageValue {
                name: "mongoose".to_string(),
                version: "1.0.0".to_string(),
                path: "".to_string(),
            }],
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
                PackageValue {
                    name: "mongoose".to_string(),
                    version: "2.0.0".to_string(),
                    path: "".to_string(),
                },
                PackageValue {
                    name: "mongoose".to_string(),
                    version: "1.0.0".to_string(),
                    path: "".to_string(),
                },
            ],
        );

        assert_eq!(hash_map, result_hash_map);
    }

    #[test]
    fn it_should_find_bad_values() {
        let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
        hash_map.insert(
            "mongoose".to_string(),
            vec![
                PackageValue {
                    name: "mongoose".to_string(),
                    version: "2.0.0".to_string(),
                    path: "path/to/mongoose".to_string(),
                },
                PackageValue {
                    name: "mongoose".to_string(),
                    version: "1.0.0".to_string(),
                    path: "path/to/mongoose".to_string(),
                },
            ],
        );

        let bad_values = find_bad_values(&hash_map);

        assert_eq!(
            bad_values,
            vec!["Package: mongoose. Unique versions: 2. Highest version: 2.0.0. Located: path/to/mongoose."]
        );
    }

    #[test]
    fn it_should_return_empty_vec_for_good_values() {
        let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
        hash_map.insert(
            "mongoose".to_string(),
            vec![PackageValue {
                name: "mongoose".to_string(),
                version: "1.0.0".to_string(),
                path: "".to_string(),
            }],
        );

        let bad_values = find_bad_values(&hash_map);

        let empty_vec: Vec<String> = Vec::new();
        assert_eq!(bad_values, empty_vec);
    }

    #[test]
    fn it_should_call_all_together() {
        let path = "./src/data/package.json".to_string();
        let result = find_duplicate_dependencies(vec![path]);

        let empty_vec: Vec<String> = Vec::new();
        assert_eq!(result, empty_vec);
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
                PackageValue {
                    name: "mongoose".to_string(),
                    version: "2.0.0".to_string(),
                    path: "".to_string(),
                },
                PackageValue {
                    name: "mongoose".to_string(),
                    version: "1.0.0".to_string(),
                    path: "".to_string(),
                },
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
            vec![PackageValue {
                name: "mongoose".to_string(),
                version: "1.0.0".to_string(),
                path: path.to_string(),
            }],
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
                PackageValue {
                    name: "mongoose".to_string(),
                    version: "2.1.1".to_string(),
                    path: "".to_string(),
                },
                PackageValue {
                    name: "mongoose".to_string(),
                    version: "2.1.0".to_string(),
                    path: "".to_string(),
                },
                PackageValue {
                    name: "mongoose".to_string(),
                    version: "2.0.0".to_string(),
                    path: "".to_string(),
                },
                PackageValue {
                    name: "mongoose".to_string(),
                    version: "1.0.0".to_string(),
                    path: "".to_string(),
                },
                PackageValue {
                    name: "mongoose".to_string(),
                    version: "2.0.1".to_string(),
                    path: "".to_string(),
                },
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
            vec![PackageValue {
                name: "mongoose".to_string(),
                version: "1.0.0".to_string(),
                path: "".to_string(),
            }],
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
                PackageValue {
                    name: "mongoose".to_string(),
                    version: "1.10.0".to_string(),
                    path: "".to_string(),
                },
                PackageValue {
                    name: "mongoose".to_string(),
                    version: "1.3.0".to_string(),
                    path: "".to_string(),
                },
            ],
        );

        assert_eq!(hash_map, result_hash_map);
    }
}
