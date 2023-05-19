fmt:
    cargo fmt
    prettier --write .
    taplo fmt *toml
    just --fmt --unstable

update:
    cargo upgrade
    cargo update

check:
    pre-commit run --all-files
    cargo deny --workspace check
    cargo outdated --workspace --exit-code 1
    cargo +nightly udeps --workspace --all-targets
    cargo clippy --workspace --all-targets -- --deny clippy::all

coverage:
    RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE="./target/debug/default-%p-%m.profraw"  cargo nextest run --workspace --all-targets
    grcov ./target/debug/ --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing --ignore "/*"  -o ./target/debug/coverage/
    open ./target/debug/coverage/index.html

build:
    cargo build --workspace --all-targets

test:
    cargo nextest run --workspace --all-targets

changelog:
    git cliff -o CHANGELOG.md
    prettier --write CHANGELOG.md
