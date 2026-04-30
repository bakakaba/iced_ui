default:
    @just --list

# Run the demo gallery
dev:
    cargo run -p demo

# Build all packages
build:
    cargo build --workspace

# Run all tests
test:
    cargo test --workspace

lint:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings

fix:
    cargo fmt --all
    cargo clippy --workspace --all-targets --fix

# Dry-run a publish of the library crate
publish-dry:
    cargo publish -p iced_ui --dry-run
