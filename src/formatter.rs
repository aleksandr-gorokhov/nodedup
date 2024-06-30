use std::collections::HashMap;
use std::marker::PhantomData;

use colored::*;

use crate::parser::PackageValue;

#[derive(Debug, PartialEq)]
enum FormatStyles {
    Default,
    Full,
    Short,
}

pub struct Empty {}
pub struct Ready {}

pub struct DependenciesFormatter<State = Empty> {
    state: PhantomData<State>,
    style: FormatStyles,
    dependencies: HashMap<String, Vec<PackageValue>>,
}

impl DependenciesFormatter<Empty> {
    pub fn new(dependencies: HashMap<String, Vec<PackageValue>>) -> DependenciesFormatter<Ready> {
        DependenciesFormatter {
            dependencies,
            style: FormatStyles::Default,
            state: PhantomData::<Ready>,
        }
    }
}

impl DependenciesFormatter<Ready> {
    pub fn try_set_style(&mut self, style: &str) {
        if ["short", "default", "full"].iter().any(|v| v == &style) {
            self.set_style(match style {
                "short" => FormatStyles::Short,
                "full" => FormatStyles::Full,
                _ => FormatStyles::Default,
            });
            return;
        }

        panic!("Unknown style format: {}", style)
    }

    fn set_style(&mut self, style: FormatStyles) {
        self.style = style
    }

    pub fn format(&self, color: bool) -> String {
        let mut formatted = String::new();

        for (name, values) in &self.dependencies {
            formatted.push_str(&format!(
                "{}, Unique versions: {}\n",
                name.red(),
                values.len().to_string().red()
            ));
            if self.style == FormatStyles::Short {
                continue;
            }
            formatted.push_str(&format!(
                "{}{}\n\n",
                "Locations:\n".green(),
                values
                    .iter()
                    .map(|v| v.path.clone())
                    .collect::<Vec<String>>()
                    .join("\n")
            ));
            if self.style == FormatStyles::Default {
                continue;
            }
            formatted.push_str(&format!(
                "{}{}\n\n",
                "Versions:\n".green(),
                values
                    .iter()
                    .map(|v| v.version.clone())
                    .collect::<Vec<String>>()
                    .join("\n")
            ));
        }

        if color {
            formatted
        } else {
            regex::Regex::new("\x1B\\[[0-9;]*m")
                .unwrap()
                .replace_all(&formatted, "")
                .to_string()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod try_set_style {
        use super::*;

        #[test]
        #[should_panic]
        fn it_should_panic_on_wrong_string() {
            let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
            hash_map.insert(
                "test".to_string(),
                vec![
                    PackageValue::new("test", "1.0.0", "./src/1"),
                    PackageValue::new("test", "2.0.0", "./src/2"),
                ],
            );

            let mut formatter = DependenciesFormatter::new(hash_map);
            formatter.try_set_style("error");
        }
    }

    mod format {
        use super::*;

        #[test]
        fn it_should_full_format_dependencies() {
            let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
            hash_map.insert(
                "test".to_string(),
                vec![
                    PackageValue::new("test", "1.0.0", "./src/1"),
                    PackageValue::new("test", "2.0.0", "./src/2"),
                ],
            );

            let mut formatter = DependenciesFormatter::new(hash_map);
            formatter.set_style(FormatStyles::Full);

            let formatted = formatter.format(false);
            assert_eq!(
                regex::Regex::new("\x1B\\[[0-9;]*m")
                    .unwrap()
                    .replace_all(&formatted, "")
                    .to_string(),
                "test, Unique versions: 2\nLocations:\n./src/1\n./src/2\n\nVersions:\n1.0.0\n2.0.0\n\n"
            );
        }

        #[test]
        fn it_should_short_format_dependencies() {
            let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
            hash_map.insert(
                "test".to_string(),
                vec![
                    PackageValue::new("test", "1.0.0", "./src/1"),
                    PackageValue::new("test", "2.0.0", "./src/2"),
                ],
            );

            let mut formatter = DependenciesFormatter::new(hash_map);
            formatter.set_style(FormatStyles::Short);

            let formatted = formatter.format(false);
            assert_eq!(
                regex::Regex::new("\x1B\\[[0-9;]*m")
                    .unwrap()
                    .replace_all(&formatted, "")
                    .to_string(),
                "test, Unique versions: 2\n"
            );
        }

        #[test]
        fn it_should_default_format_dependencies() {
            let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
            hash_map.insert(
                "test".to_string(),
                vec![
                    PackageValue::new("test", "1.0.0", "./src/1"),
                    PackageValue::new("test", "2.0.0", "./src/2"),
                ],
            );

            let formatter = DependenciesFormatter::new(hash_map);

            let formatted = formatter.format(false);
            assert_eq!(
                regex::Regex::new("\x1B\\[[0-9;]*m")
                    .unwrap()
                    .replace_all(&formatted, "")
                    .to_string(),
                "test, Unique versions: 2\nLocations:\n./src/1\n./src/2\n\n"
            );
        }

        mod color {
            use super::*;

            #[test]
            fn it_should_keep_color() {
                let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
                hash_map.insert(
                    "test".to_string(),
                    vec![
                        PackageValue::new("test", "1.0.0", "./src/1"),
                        PackageValue::new("test", "2.0.0", "./src/2"),
                    ],
                );

                let formatter = DependenciesFormatter::new(hash_map);

                let formatted = formatter.format(true);
                assert_ne!(
                    formatted,
                    "test, Unique versions: 2\nLocations:\n./src/1\n./src/2\n\n"
                );
            }

            #[test]
            fn it_should_remove_color() {
                let mut hash_map: HashMap<String, Vec<PackageValue>> = HashMap::new();
                hash_map.insert(
                    "test".to_string(),
                    vec![
                        PackageValue::new("test", "1.0.0", "./src/1"),
                        PackageValue::new("test", "2.0.0", "./src/2"),
                    ],
                );

                let formatter = DependenciesFormatter::new(hash_map);

                let formatted = formatter.format(false);
                assert_eq!(
                    formatted,
                    "test, Unique versions: 2\nLocations:\n./src/1\n./src/2\n\n"
                );
            }
        }
    }
}
