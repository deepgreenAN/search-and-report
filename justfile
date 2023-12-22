set shell := ["nu", "-c"]

default: run

build:
    cargo build --release

run:
    ./target/release/main -i