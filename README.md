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
  -s, --silent           Exit with zero code when duplicates are found
  -h, --help             Print help
  -V, --version          Print version