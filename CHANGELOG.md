# Changelog

All notable changes to this project will be documented in this file.

## [0.7.2] - 2024-04-17

### 🐛 Bug Fixes

- _(ci)_ Add semver-checks to .gitignore
- _(ciweimao)_ Downgrade app version

### ⚙️ Miscellaneous Tasks

- _(ci)_ Remove unnecessary operations

## [0.7.1] - 2024-04-13

### 🐛 Bug Fixes

- _(ciyuanji)_ Some chapters failed to download

### 📚 Documentation

- Update README.md

## [0.7.0] - 2024-04-12

### ⛰️ Features

- _(download)_ Chapter download failure no longer exits the program

### 🐛 Bug Fixes

- _(search)_ Limit search size

## [0.6.0] - 2024-04-11

### ⛰️ Features

- _(check)_ Add basic_check flag
- _(check)_ Consider the metadata.lang attribute
- _(transform)_ Better image convert
- Add template command
- _(check)_ Allow input directory
- Support ciyuanji

### 🐛 Bug Fixes

- _(transform)_ Title is missing spaces
- Saving webp images uses lossy encoding
- _(info)_ Ciyuanji need login
- Fix parse markdown failed on windows again
- Fix parse markdown failed on windows
- Fix ambiguous path display

### 🚜 Refactor

- Refactor code
- Refactor code and update dependencies
- Use testresult
- Some small improvements
- Remove some dead code
- Some small improvements
- Simplify download and generated code
- _(build)_ Some small improvements
- _(bookshelf)_ Some small improvements
- _(check,transform)_ Upgrade pulldown-cmark and use new features

### 🧪 Testing

- Fix transform test

### ⚙️ Miscellaneous Tasks

- _(locales)_ Update translation
- Change opt-level when dev
- Upgrade various project files
- Rename commitlint.config.js
- Upgrade various project files
- Remove a todo
- _(ci)_ Fix install pandoc for macOS
- Upgrade various project files

## [0.5.0] - 2023-07-31

### ⛰️ Features

- Improve user name and password enter

### 🚜 Refactor

- Modify translation
- Use dialoguer
- Change time-consuming log level
- Use the latest novel-api
- Use mimalloc
- Use hashbrown

### 🧪 Testing

- Rename some test
- Improve test parallelism

### ⚙️ Miscellaneous Tasks

- No longer ignore RUSTSEC-2022-0090

## [0.4.0] - 2023-07-11

### ⛰️ Features

- _(transform)_ Convert Image Format
- _(transform)_ Remove extra line breaks
- Improve conversion function

### 🐛 Bug Fixes

- _(transform)_ Guaranteed to end with a newline character
- _(transform)_ Does not serialize Option::None

### 🚜 Refactor

- Use open instead of opener

### ⚙️ Miscellaneous Tasks

- _(ci)_ Bump pandoc

## [0.3.8] - 2023-06-12

### ⛰️ Features

- Improve version info
- Colored help message

### 🐛 Bug Fixes

- _(sfacg)_ Fixes #98

### 🧪 Testing

- Add help test
- Add version test

### ⚙️ Miscellaneous Tasks

- _(ci)_ Bump pandoc from 3.1.2 to 3.1.3

## [0.3.7] - 2023-06-05

### ⚙️ Miscellaneous Tasks

- No longer include debug information

## [0.3.6] - 2023-06-05

### ⛰️ Features

- Add --skip-login option
- Add os info for -V
- Use human-panic
- Add backtrace option
- Handle C Locale

### 🐛 Bug Fixes

- Do not panic when get terminal size failed
- _(check)_ Set working directory using absolute path

### 🚜 Refactor

- Remove human_panic
- Bump novel-api to 0.7.1
- Use std::io::IsTerminal
- _(update)_ No longer use spawn_blocking
- Use color-eyre instead of anyhow

### 📚 Documentation

- Add msrv badge
- Update README.md

### 🧪 Testing

- Delete update test due to network problems
- Add --backtrace
- Add --backtrace
- Add download test
- Add update test

### ⚙️ Miscellaneous Tasks

- _(publish)_ Do not run strip
- _(publish)_ Add pdb file on windows
- _(publish)_ Add pdb file on windows
- _(publish)_ Add pdb file on windows
- _(publish)_ Add pdb file on windows
- Generates the minimal amount of debug info
- Correct incorrect manifest field
- Downgrade enum-as-inner
- Update .justfile
- _(ci)_ Do not set release body on windows
- Update cliff.toml
- Update cliff.toml
- Update Cargo.lock
- Update changelog
- Update cliff.toml
- Update changelog
- _(ci)_ Use upx on x86_64-apple-darwin
- Use cargo-nextest
- Add pre-commit check
- Update changelog
- _(ci)_ Change release body

## [0.2.1] - 2023-05-18

### ⛰️ Features

- _(info)_ Add content preview function
- Complete search subcommand
- Preliminary addition of library function
- Preliminary addition of library function
- Update unicode to version 15

### 🐛 Bug Fixes

- Fix directory deletion failure on windows
- Use dunce to avoid path error on windows
- _(transform)_ Wrong link break replace
- Use windows line break to write file
- Fix windows build failed
- Windows link break verify
- _(check)_ Handle Windows line breaks
- _(pandoc)_ Fixes #63
- Strip extra characters on Windows terminal
- _(favorites)_ Fixes #48

### 🚜 Refactor

- Some minor modifications
- Some minor modifications
- Some minor modifications
- _(search)_ Optimize code logic
- Rename favorites to bookshelf
- _(read)_ Optimize code logic
- Use windows line break in windows output
- _(info)_ Optimize code logic
- Some minor modifications
- _(transform)_ Optimize code logic
- _(unzip)_ Optimize code logic
- _(zip)_ Optimize code logic
- Use try_exists
- _(check)_ Optimize code logic
- _(build)_ Optimize code logic
- _(pandoc)_ Fixes #77
- _(pandoc)_ Use commonmark_x input format
- Some minor modifications
- Improve locale recognition
- Some minor modifications
- Many minor modifications
- Many minor modifications
- Many minor modifications
- Ignore image download failed
- Change source display name

### 📚 Documentation

- Update README.md
- Update README.md
- Update README.md
- Update README.md
- Add command-line usage instructions and examples ([#62](https://github.com/novel-rs/cli/issues/62))
- Update README.md
- Add changelog

### ⚡ Performance

- _(build)_ Optimize file deletion
- Optimize the perf of verify_line_break

### 🎨 Styling

- Run rustfmt
- Run rustfmt

### 🧪 Testing

- Add search test
- Simplify the code
- Better transform test
- _(info)_ Add test
- Fix wrong test
- _(transform)_ Add test
- _(zip)_ Add test
- _(unzip)_ Add test
- Better link break test
- _(completions)_ Add test
- _(check)_ Add test
- _(build)_ Add test

### ⚙️ Miscellaneous Tasks

- Update Cargo.lock
- _(ci)_ Run git-cliff only on ubuntu
- _(ci)_ Run rm only on ubuntu
- Exclude test data when publish
- Update changelog
- _(ci)_ Rename some steps
- _(ci)_ Remove outdated action
- _(ci)_ Specify nasm version
- Add .gitattributes file
- _(ci)_ Use brew install pandoc on ubuntu
- Fix windows build CI
- Fix CI windows install pandoc
- Change cliff.toml
- Add git-cliff to generate changelog
- Ignore RUSTSEC-2022-0040
- Run upx before release
- Disable default-features for all crates
- Add check semver
- Remove outdated action schedule
- Add aarch64-apple-darwin target

## [0.1.4] - 2023-01-27

### ⛰️ Features

- Initial

### 🐛 Bug Fixes

- Wrong help message
- Publish failed

### 🚜 Refactor

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

### ⚙️ Miscellaneous Tasks

- Move profile.release to Cargo.toml
- Add archive compress level
- Change zip path
- Add executable file permission
- Archive release file
- Bump version

### Doc

- Update README.md
