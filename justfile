# This file is only for development
set windows-shell := ["powershell.exe"]
set dotenv-load := true
default: test

run:
  cargo run -- -f txt,json,csv --images

debug:
  cargo run -- -f txt,json,csv --images --debug

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