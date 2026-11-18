KERNEL_PATH := ../..

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

# Test that code examples compile. Build both crates in one cargo invocation so the
# resulting target/debug/deps contains a single buoyant_kernel rmeta with the unified
# feature set. Two separate cargo builds would leave two rmetas behind and mdbook test
# cannot pick one (E0464).
test:
	cargo clean --manifest-path $(KERNEL_PATH)/kernel/Cargo.toml
	cargo build --locked --manifest-path $(KERNEL_PATH)/Cargo.toml \
	  -p buoyant_kernel -p buoyant_kernel_engine \
	  --features "buoyant_kernel/internal-api,buoyant_kernel/prettyprint,buoyant_kernel_engine/rustls,buoyant_kernel_engine/arrow"
	rustup run $(RUST_TOOLCHAIN) mdbook test -L $(KERNEL_PATH)/target/debug/deps

# Clean build artifacts
clean:
	rustup run $(RUST_TOOLCHAIN) mdbook clean
