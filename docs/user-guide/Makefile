KERNEL_PATH := ../..
KERNEL_FEATURES := default-engine-rustls,arrow,internal-api,prettyprint

# mdbook ignores rust-toolchain.toml so we have to use rustup to run
# with the correct toolchain version (https://github.com/rust-lang/mdBook/issues/2501)
RUST_TOOLCHAIN := 1.91

.PHONY: test serve build clean

# Build the book
build:
	rustup run $(RUST_TOOLCHAIN) mdbook build

# Serve locally with hot reload
serve:
	rustup run $(RUST_TOOLCHAIN) mdbook serve

# Test that code examples compile
test:
	cargo clean --manifest-path $(KERNEL_PATH)/kernel/Cargo.toml
	cargo build --locked --manifest-path $(KERNEL_PATH)/kernel/Cargo.toml --features "$(KERNEL_FEATURES)"
	rustup run $(RUST_TOOLCHAIN) mdbook test -L $(KERNEL_PATH)/target/debug/deps

# Clean build artifacts
clean:
	rustup run $(RUST_TOOLCHAIN) mdbook clean
