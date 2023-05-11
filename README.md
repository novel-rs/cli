# cli

[![Build](https://github.com/novel-rs/cli/actions/workflows/build.yml/badge.svg)](https://github.com/novel-rs/cli/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/novel-rs/cli/branch/main/graph/badge.svg?token=96TJ1OIF3P)](https://codecov.io/gh/novel-rs/cli)
[![docs.rs](https://img.shields.io/docsrs/novel-cli)](https://docs.rs/novel-cli)
[![Crates.io](https://img.shields.io/crates/l/novel-cli)](https://github.com/novel-rs/cli)
[![Crates.io](https://img.shields.io/crates/v/novel-cli)](https://crates.io/crates/novel-cli)

---

A set of tools for downloading novels from the web, manipulating text, and generating EPUB

## Platform

- Windows
- Linux
- macOS

## Installation

You can download the compiled file from [release](https://github.com/novel-rs/cli/releases), or compile it yourself

```shell
cargo install novel-cli
```

If you compile it yourself, you need the following dependencies:

- Clang
- CMake
- Python / Python3

If you are using Windows, you also need:

- NASM

The **novel-cli build** subcommand requires [pandoc](https://github.com/jgm/pandoc)

The **novel-cli real-cugan** subcommand requires [realcugan-ncnn-vulkan](https://github.com/nihui/realcugan-ncnn-vulkan)

## Usage

You can run `novel-cli help` to view help information

Note that markdown file downloaded by `novel-cli download --format=pandoc` often need to be manually modified before using `novel-cli build` to create epub.

### Examples

- **The basic format of the command is:**

```shell
novel-cli [OPTIONS] <COMMAND> [COMMAND-OPTIONS] [ARGUMENTS]
```

- **Download a novel from source in format**

```shell
novel-cli download <bookid> --source <app-source> --format <output-format>
```

- **Search for a novel from source**

```shell
novel-cli search --source <SOURCE> <KEYWORD>
```

### Commands

- `download`: Download novels from various sources.
- `search`: Search for novels on various sources.
- `info`: Print information about a novel on various sources.
- `read`: Read novel from various sources.
- `bookshelf`: Show saved favorite novels on various sources.
- `transform`: Convert markdown files to pandoc style.
- `check`: Check the format and content of pandoc style markdown files.
- `build`: Build a novel from pandoc style markdown files or an mdBook folder.
- `zip`: Compress an epub folder.
- `unzip`: Decompress an epub file.
- `real-cugan`: Run the realcugan-ncnn-vulkan program.
- `update`: Check for updates, download files from GitHub, and replace them.
- `completions`: Generate shell completions to standard output.
- `help`: Print this message or the help of the given subcommand(s).

### Options

- `-v, --verbose`: Use verbose output. This option provides more detailed output.
- `-q, --quiet`: Do not print logs (default: **false**). This option suppresses logging output.
- `-h, --help`: Print help. This option displays the help information.
- `-V, --version`: Print version. This option prints the version information.

## Contributing

You should read [CONTRIBUTING](https://github.com/novel-rs/cli/blob/main/CONTRIBUTING.md) first

## License

All the code in this repository is released under **[Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)**
and **[MIT license](https://opensource.org/licenses/MIT)**
