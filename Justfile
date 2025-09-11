# Default: build everything in debug mode
default:
    just build-all

# --------------------
# Build Commands
# --------------------

# Debug + release workspace
build-all:
    cargo build --workspace
build-all-release:
    cargo build --workspace --release

# Core crate
build-core:
    cargo build -p core
build-core-release:
    cargo build -p core --release

# Desktop (Linux native debug + release)
build-desktop:
    cargo build -p desktop
build-desktop-release:
    cargo build -p desktop --release

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
run: run-desktop

run-desktop:
    cargo run -p desktop

run-desktop-release:
    cargo run -p desktop --release

# --------------------
# Tests
# --------------------

test: test-all

test-all:
    cargo test --workspace
test-core:
    cargo test -p core --lib --quiet
test-desktop:
    cargo test -p desktop --quiet