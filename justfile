# This file is only for development
set windows-shell := ["powershell.exe"]
set dotenv-load := true
default: test

messages:
  cargo run -- messages -f txt,json,csv --images --no-usernames --redact --token

messages-dbg:
  cargo run -- messages -f json --debug --images --token

subreddit:
  cargo run -- subreddit -n r/rexitTest -f txt,json,csv --images

subreddit-dbg:
  cargo run -- subreddit -n r/rexitTest -f txt,json,csv --images --debug

saved:
  cargo run -- saved -f txt,json,csv --images

saved-dbg:
  cargo run -- saved -f txt,json,csv --images --debug

test:
    cargo test

test-creds:
    cargo test -- --include-ignored

doc: 
    cargo doc --no-deps --open

codecov:
    $env:RUSTFLAGS="-Cinstrument-coverage"; cargo build --profile codecov
    LLVM_PROFILE_FILE="your_name-%p-%m.profraw" RUSTFLAGS="-Cinstrument-coverage" cargo test -j 1 --all-features --profile codecov
    grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./codecov

# Bump the version, commit, tag and push; CI then builds the binaries and publishes to crates.io
release version:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "$(git status --porcelain)" ]; then echo "Working tree not clean; commit first"; exit 1; fi
    sed -i.bak 's/^version = ".*"/version = "{{version}}"/' Cargo.toml && rm Cargo.toml.bak
    cargo check --quiet
    git commit -am "Release v{{version}}"
    git tag "v{{version}}"
    git push origin main "v{{version}}"
