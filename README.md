# rdgrep

[![GitHub Actions CI](https://github.com/craftscat/rdgrep/workflows/CI/badge.svg)](https://github.com/craftscat/rdgrep/actions?query=workflow%3ACI)

`rdgrep` is a tool to display the number of files that have disabled [RuboCop](https://github.com/rubocop/rubocop) conventions(`rubocop:disable`).

## Installation

### Using cargo

```bash
$ cargo install rdgrep
```

### From repository relases page

https://github.com/craftscat/rdgrep/releases


## Usage

Scan the `rb` files under the specified directory and print the number of files that have been disabled by convention.

```bash
$ rdgrep ./
("Style/AccessModifierDeclarations", 3)
("Style/Alias", 2)
("Style/AccessorGrouping", 1)
```
