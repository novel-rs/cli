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

If you are using Windows, you also need:

- NASM

The **novel-cli build** subcommand requires [pandoc](https://github.com/jgm/pandoc)

The **novel-cli real-cugan** subcommand requires [realcugan-ncnn-vulkan](https://github.com/nihui/realcugan-ncnn-vulkan)

## Usage

You can run **novel-cli help** to view help information

## Contributing

You should read [CONTRIBUTING](https://github.com/novel-rs/cli/blob/main/CONTRIBUTING.md) first

## License

All the code in this repository is released under **[Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)** and **[MIT license](https://opensource.org/licenses/MIT)**
