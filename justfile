# This file is only for development
set windows-shell := ["powershell.exe"]

default: test

test:
    cargo test

test-creds:
    cargo test -- --include-ignored

doc: 
    cargo doc --no-deps --open