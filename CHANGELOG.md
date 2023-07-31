# Changelog

All notable changes to this project will be documented in this file.

## [0.5.0] - 2023-07-31

### <!-- 0 -->⛰️ Features

- Improve user name and password enter

### <!-- 2 -->🚜 Refactor

- Modify translation
- Use dialoguer
- Change time-consuming log level
- Use the latest novel-api
- Use mimalloc
- Use hashbrown

### <!-- 6 -->🧪 Testing

- Rename some test
- Improve test parallelism

### <!-- 7 -->⚙️ Miscellaneous Tasks

- No longer ignore RUSTSEC-2022-0090
- Update deps
- Bump tempfile from 3.6.0 to 3.7.0 ([#111](https://github.com/novel-rs/cli/issues/111))
- Bump serde_yaml from 0.9.24 to 0.9.25 ([#112](https://github.com/novel-rs/cli/issues/112))
- Bump clap from 4.3.17 to 4.3.19 ([#109](https://github.com/novel-rs/cli/issues/109))
- Bump serde from 1.0.173 to 1.0.175 ([#110](https://github.com/novel-rs/cli/issues/110))
- Update deps
- Bump serde_yaml from 0.9.23 to 0.9.24 ([#106](https://github.com/novel-rs/cli/issues/106))
- Bump serde_with from 3.0.0 to 3.1.0 ([#107](https://github.com/novel-rs/cli/issues/107))
- Bump clap from 4.3.12 to 4.3.14 ([#108](https://github.com/novel-rs/cli/issues/108))
- Update deps

## [0.4.0] - 2023-07-11

### <!-- 0 -->⛰️ Features

- Convert Image Format
- Remove extra line breaks
- Improve conversion function

### <!-- 1 -->🐛 Bug Fixes

- Guaranteed to end with a newline character
- Does not serialize Option::None

### <!-- 2 -->🚜 Refactor

- Use open instead of opener

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Bump pandoc
- Update deps
- Update deps
- Bump clap from 4.3.9 to 4.3.10 ([#102](https://github.com/novel-rs/cli/issues/102))
- Bump serde from 1.0.164 to 1.0.166 ([#103](https://github.com/novel-rs/cli/issues/103))
- Bump trash from 3.0.3 to 3.0.4 ([#104](https://github.com/novel-rs/cli/issues/104))
- Update deps
- Update deps
- Pre-commit autoupdate ([#100](https://github.com/novel-rs/cli/issues/100))
- Update deps
- Pre-commit autoupdate ([#99](https://github.com/novel-rs/cli/issues/99))

## [0.3.8] - 2023-06-12

### <!-- 0 -->⛰️ Features

- Improve version info
- Colored help message

### <!-- 1 -->🐛 Bug Fixes

- Fixes #98

### <!-- 6 -->🧪 Testing

- Add help test
- Add version test

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Bump pandoc from 3.1.2 to 3.1.3
- Update deps
- Bump self_update from 0.36.0 to 0.37.0 ([#94](https://github.com/novel-rs/cli/issues/94))
- Bump clap from 4.3.1 to 4.3.2 ([#97](https://github.com/novel-rs/cli/issues/97))
- Bump url from 2.3.1 to 2.4.0 ([#96](https://github.com/novel-rs/cli/issues/96))
- Bump regex from 1.8.3 to 1.8.4 ([#95](https://github.com/novel-rs/cli/issues/95))

## [0.3.7] - 2023-06-05

### <!-- 7 -->⚙️ Miscellaneous Tasks

- No longer include debug information

## [0.3.6] - 2023-06-05

### <!-- 0 -->⛰️ Features

- Add --skip-login option
- Add os info for -V
- Use human-panic
- Add backtrace option
- Handle C Locale

### <!-- 1 -->🐛 Bug Fixes

- Do not panic when get terminal size failed
- Set working directory using absolute path

### <!-- 2 -->🚜 Refactor

- Remove human_panic
- Bump novel-api to 0.7.1
- Use std::io::IsTerminal
- No longer use spawn_blocking
- Use color-eyre instead of anyhow

### <!-- 3 -->📚 Documentation

- Add msrv badge
- Update README.md

### <!-- 6 -->🧪 Testing

- Delete update test due to network problems
- Add --backtrace
- Add --backtrace
- Add download test
- Add update test

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Do not run strip
- Add pdb file on windows
- Add pdb file on windows
- Add pdb file on windows
- Add pdb file on windows
- Generates the minimal amount of debug info
- Correct incorrect manifest field
- Update deps
- Record minimum supported Rust version in metadata ([#93](https://github.com/novel-rs/cli/issues/93))
- Bump indicatif from 0.17.3 to 0.17.4 ([#89](https://github.com/novel-rs/cli/issues/89))
- Bump once_cell from 1.17.1 to 1.17.2 ([#90](https://github.com/novel-rs/cli/issues/90))
- Downgrade enum-as-inner
- Update deps
- Update .justfile
- Do not set release body on windows
- Update cliff.toml
- Update cliff.toml
- Update Cargo.lock
- Update changelog
- Update cliff.toml
- Update changelog
- Update deps
- Use upx on x86_64-apple-darwin
- Bump regex from 1.8.1 to 1.8.2 ([#86](https://github.com/novel-rs/cli/issues/86))
- Update deps
- Update deps
- Use cargo-nextest
- Add pre-commit check
- Update changelog
- Change release body

## [0.2.1] - 2023-05-18

### <!-- 0 -->⛰️ Features

- Add content preview function
- Complete search subcommand
- Preliminary addition of library function
- Preliminary addition of library function
- Update unicode to version 15

### <!-- 1 -->🐛 Bug Fixes

- Fix directory deletion failure on windows
- Use dunce to avoid path error on windows
- Wrong link break replace
- Use windows line break to write file
- Fix windows build failed
- Windows link break verify
- Handle Windows line breaks
- Fixes #63
- Strip extra characters on Windows terminal
- Fixes #48

### <!-- 2 -->🚜 Refactor

- Some minor modifications
- Some minor modifications
- Some minor modifications
- Optimize code logic
- Rename favorites to bookshelf
- Optimize code logic
- Use windows line break in windows output
- Optimize code logic
- Some minor modifications
- Optimize code logic
- Optimize code logic
- Optimize code logic
- Use try_exists
- Optimize code logic
- Optimize code logic
- Fixes #77
- Use commonmark_x input format
- Some minor modifications
- Improve locale recognition
- Some minor modifications
- Many minor modifications
- Many minor modifications
- Many minor modifications
- Ignore image download failed
- Change source display name

### <!-- 3 -->📚 Documentation

- Update README.md
- Update README.md
- Update README.md
- Update README.md
- Add command-line usage instructions and examples ([#62](https://github.com/novel-rs/cli/issues/62))
- Update README.md
- Add changelog

### <!-- 4 -->⚡ Performance

- Optimize file deletion
- Optimize the perf of verify_line_break

### <!-- 5 -->🎨 Styling

- Run rustfmt
- Run rustfmt

### <!-- 6 -->🧪 Testing

- Add search test
- Simplify the code
- Better transform test
- Add test
- Fix wrong test
- Add test
- Add test
- Add test
- Better link break test
- Add test
- Add test
- Add test

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Update Cargo.lock
- Run git-cliff only on ubuntu
- Run rm only on ubuntu
- Exclude test data when publish
- Update changelog
- Update deps
- Rename some steps
- Remove outdated action
- Specify nasm version
- Update deps
- Update deps
- Add .gitattributes file
- Update deps
- Update deps
- Use brew install pandoc on ubuntu
- Fix windows build CI
- Fix CI windows install pandoc
- Update deps
- Change cliff.toml
- Update deps
- Add git-cliff to generate changelog
- Ignore RUSTSEC-2022-0040
- Bump h2 from 0.3.16 to 0.3.17 ([#64](https://github.com/novel-rs/cli/issues/64))
- Run upx before release
- Update deps
- Update deps
- Update deps
- Update deps
- Pre-commit autoupdate ([#54](https://github.com/novel-rs/cli/issues/54))
- Bump clap_complete from 4.1.6 to 4.2.0 ([#55](https://github.com/novel-rs/cli/issues/55))
- Bump clap from 4.1.14 to 4.2.1 ([#56](https://github.com/novel-rs/cli/issues/56))
- Bump serde from 1.0.158 to 1.0.159 ([#58](https://github.com/novel-rs/cli/issues/58))
- Bump is-terminal from 0.4.5 to 0.4.6 ([#57](https://github.com/novel-rs/cli/issues/57))
- Update deps
- Bump bat from 0.22.1 to 0.23.0 ([#50](https://github.com/novel-rs/cli/issues/50))
- Bump regex from 1.7.1 to 1.7.3 ([#49](https://github.com/novel-rs/cli/issues/49))
- Bump opener from 0.5.2 to 0.6.0 ([#51](https://github.com/novel-rs/cli/issues/51))
- Bump image from 0.24.5 to 0.24.6 ([#52](https://github.com/novel-rs/cli/issues/52))
- Bump tokio from 1.26.0 to 1.27.0 ([#53](https://github.com/novel-rs/cli/issues/53))
- Bump clap from 4.1.9 to 4.1.11 ([#44](https://github.com/novel-rs/cli/issues/44))
- Bump is-terminal from 0.4.4 to 0.4.5 ([#46](https://github.com/novel-rs/cli/issues/46))
- Bump novel-api from `2875ffb` to `f5d30ed` ([#45](https://github.com/novel-rs/cli/issues/45))
- Bump anyhow from 1.0.69 to 1.0.70 ([#43](https://github.com/novel-rs/cli/issues/43))
- Bump serde from 1.0.156 to 1.0.158 ([#42](https://github.com/novel-rs/cli/issues/42))
- Pre-commit autoupdate ([#41](https://github.com/novel-rs/cli/issues/41))
- Update deps
- Update deps
- Update deps
- Bump opencc-rs from `3df551d` to `53be6cd` ([#29](https://github.com/novel-rs/cli/issues/29))
- Bump novel-api
- Update deps
- Bump thedoctor0/zip-release from 0.7.1 to 0.7.2 ([#28](https://github.com/novel-rs/cli/issues/28))
- Pre-commit autoupdate ([#23](https://github.com/novel-rs/cli/issues/23))
- Bump novel-api from `a4e230b` to `c0f8a13` ([#9](https://github.com/novel-rs/cli/issues/9))
- Update deps
- Bump is-terminal from 0.4.2 to 0.4.3 ([#7](https://github.com/novel-rs/cli/issues/7))
- Bump fs_extra from 1.2.0 to 1.3.0 ([#8](https://github.com/novel-rs/cli/issues/8))
- Update deps
- Update deps
- Use the latest mdbook
- Disable default-features for all crates
- Bump thedoctor0/zip-release from 0.7.0 to 0.7.1 ([#2](https://github.com/novel-rs/cli/issues/2))
- Bump toml from 0.7.0 to 0.7.1 ([#3](https://github.com/novel-rs/cli/issues/3))
- Update deps
- Add check semver
- Remove outdated action schedule
- Add aarch64-apple-darwin target
- Bump toml from 0.6.0 to 0.7.0 ([#1](https://github.com/novel-rs/cli/issues/1))

## [0.1.4] - 2023-01-27

### <!-- 0 -->⛰️ Features

- Initial

### <!-- 1 -->🐛 Bug Fixes

- Wrong help message
- Publish failed

### <!-- 2 -->🚜 Refactor

- Some minor modifications
- Many minor modifications
- Remove unnecessary String construction
- Optimize some logs
- Change verbose range
- Adjust command line help information
- Disable tracing_subscriber time
- Apply clippy
- Better LanguageIdentifier parse
- Don't use ripunzip use zip

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Update deps
- Move profile.release to Cargo.toml
- Add archive compress level
- Change zip path
- Add executable file permission
- Archive release file
- Update deps
- Bump version

### Doc

- Update README.md
