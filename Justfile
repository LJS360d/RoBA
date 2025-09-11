# TODO adapt for non linux platforms

default:
    just build-all

# --------------------
# Build Commands
# --------------------

alias build := build-all
alias b := build-all

build-all:
    cargo build --workspace
build-all-release:
    cargo build --workspace --release

bundle:
    cd frontends/desktop && cargo bundle --target x86_64-unknown-linux-gnu

run-bundle: bundle
    ./target/x86_64-unknown-linux-gnu/debug/bundle/appimage/desktop_*.AppImage

# WASM build
# Requires: rustup target add wasm32-unknown-unknown
build-wasm:
    cargo build -p wasm --target wasm32-unknown-unknown
build-wasm-release:
    cargo build -p wasm --target wasm32-unknown-unknown --release

# --------------------
# Run Commands
# --------------------

# Alias for run-desktop
alias run := run-desktop
alias r := run-desktop

run-desktop:
    cargo run -p desktop

run-desktop-release:
    cargo run -p desktop --release

# --------------------
# Tests
# --------------------

alias test := test-all
alias t := test-all

test-all:
    cargo test --workspace
test-core:
    cargo test -p core --lib --quiet
test-desktop:
    cargo test -p desktop --quiet