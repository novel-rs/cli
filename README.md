# cli

[![Build](https://github.com/novel-rs/cli/actions/workflows/build.yml/badge.svg)](https://github.com/novel-rs/cli/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/novel-rs/cli/branch/main/graph/badge.svg?token=96TJ1OIF3P)](https://codecov.io/gh/novel-rs/cli)
[![docs.rs](https://img.shields.io/docsrs/novel-cli)](https://docs.rs/novel-cli)
[![MSRV](https://img.shields.io/badge/rustc-1.70+-blue.svg)](https://github.com/rust-lang/rust)
[![Crates.io](https://img.shields.io/crates/l/novel-cli)](https://github.com/novel-rs/cli)
[![Crates.io](https://img.shields.io/crates/v/novel-cli)](https://crates.io/crates/novel-cli)
[![GitHub Downloads](https://img.shields.io/github/downloads/novel-rs/cli/total)](https://github.com/novel-rs/cli/releases)

---

A set of tools for downloading novels from the web, manipulating text, and generating EPUB

## Platform

- Windows
- Linux
- macOS

## Installation

You can download the compiled file from [release](https://github.com/novel-rs/cli/releases), or compile it yourself

```shell
# Download the source code from https://crates.io
cargo install novel-cli

# Or download the source code from GitHub
git clone https://github.com/novel-rs/cli
cd cli
cargo build --release
```

If you compile it yourself, you need the following dependencies:

- Clang
- CMake
- Python / Python3

The **novel-cli build** subcommand requires [pandoc](https://github.com/jgm/pandoc)

The **novel-cli real-cugan** subcommand requires [realcugan-ncnn-vulkan](https://github.com/nihui/realcugan-ncnn-vulkan)

## Usage

You can run `novel-cli help` to view help information

### Examples

- **The basic format of the command is:**

```shell
novel-cli [OPTIONS] <COMMAND> [COMMAND-OPTIONS] [ARGUMENTS]
```

- **Download a novel from source in format**

```shell
novel-cli download <novel_id> --source <source> --format <output_format>
```

- **Search for a novel from source**

```shell
novel-cli search --source <source> <keyword>
```

### Commands

- `sign`: Sign in and display the current amount of money
- `download`: Download novel
- `search`: Search novels
- `info`: Show novel information
- `read`: Read novel
- `bookshelf`: Show novels in the bookshelf
- `template`: Generate pandoc-style markdown file template
- `transform`: Transform pandoc-style markdown file
- `check`: Check pandoc-style markdown file format and content
- `build`: Build novel from pandoc-style markdown file or mdBook folder
- `zip`: Zip an EPUB folder
- `unzip`: Unzip an EPUB file
- `real-cugan`: Run realcugan-ncnn-vulkan to super-resolution images
- `update`: Check for updates, download file from GitHub, and replace
- `completions`: Generate shell completion to stdout
- `help`: Print this message or the help of the given subcommand(s)

### Options

- `-v, --verbose`: Use verbose output. This option provides more detailed output
- `-q, --quiet`: Do not print logs (default: **false**). This option suppresses logging output
- `--backtrace`: Print backtrace information. This option displays the backtrace information
- `-h, --help`: Print help. This option displays the help information
- `-V, --version`: Print version. This option prints the version information

## Contributing

You should read [CONTRIBUTING](https://github.com/novel-rs/cli/blob/main/CONTRIBUTING.md) first

## License

All the code in this repository is released under **[Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)**
and **[MIT license](https://opensource.org/licenses/MIT)**
