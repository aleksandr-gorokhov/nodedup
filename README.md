# NoDEdup: Duplicate Package Finder

NoDEdup is a CLI tool designed to help you identify duplicate JavaScript dependencies in your project. It scans all
package.json files within the specified folder, pinpointing any dependencies or devDependencies that exist in multiple
versions, helping you streamline your project's dependency tree.

# Installation

1. Make sure you
   have [cargo](https://doc.rust-lang.org/book/ch01-01-installation.html#installing-rustup-on-linux-or-macos) installed
2. ```cargo install nodedup```

# Usage

```
Usage: nodedup [OPTIONS] --folder <FOLDER>

Options:
  -f, --folder <FOLDER>  Folder to scan
  -o, --output <OUTPUT>  Output format. Possible values: 'default', 'short', 'full' [default: default]
  -s, --silent           Exit with zero code when duplicates are found
  -c, --color            Color important output
  -h, --help             Print help
  -V, --version          Print version
```

# Ignore dependencies

You can create `.ndignore` file in the root of your project to ignore specific dependencies. Each line in the file
should
be a dependency name.
Ignoring dependencies is useful when you have a dependency that you know is duplicated but you don't want to remove it.

## Sample .ndignore file

```
lodash
react
```