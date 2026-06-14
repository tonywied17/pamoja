# zero-edge workflows. Run `just` to list available recipes.

# show all recipes
default:
    @just --list

# install required toolchain components
setup:
    rustup component add rustfmt clippy

# format the whole workspace
fmt:
    cargo fmt --all

# check formatting without writing changes
fmt-check:
    cargo fmt --all -- --check

# type-check the workspace
check:
    cargo check --workspace --all-targets

# lint with clippy, warnings treated as errors
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# run the test suite
test:
    cargo test --workspace

# build the workspace
build:
    cargo build --workspace

# run everything CI runs
ci: fmt-check lint test

# regenerate language bindings (stub during Phase 0)
bindings:
    cargo xtask codegen

# publish all packages (stub during Phase 0)
release:
    cargo xtask release
