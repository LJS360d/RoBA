# Build the entire workspace
build-all:
	cargo build --workspace

# Build individual crates
build-core:
	cargo build -p core

build-desktop:
	cargo build -p desktop

# Requires the wasm32 target installed: rustup target add wasm32-unknown-unknown
build-wasm:
	cargo build -p wasm --target wasm32-unknown-unknown

# Run desktop binary
run-desktop:
	cargo run -p desktop

# Test the entire workspace
test-all:
	cargo test --workspace

# Test core crate only
test-core:
	cargo test -p core --lib --quiet
