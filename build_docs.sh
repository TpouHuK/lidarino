#!/bin/bash

# Compile docs the way i like it =)
# Add `--open` flag for opening the docs in browser.
cargo doc --no_deps --document-private-items $@

# To build docs and open them in browser, without dependencies:
# `cargo doc --open --no-deps --document-private-items`
# Or with dependencies if you want to read documentation on other crates.
# `cargo doc --open --document-private-items`
