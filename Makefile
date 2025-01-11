MAKEFLAGS += -j4
.PHONY: *
export RUST_LOG=info

test:
	cargo test

build:
	cd codebook-lsp && cargo build

build-release:
	cd codebook-lsp && cargo build --release

integration_test: build
	cd integration_tests && bun test

install_ext: build-release
	cp -f target/release/codebook-lsp "${HOME}/Library/Application Support/Zed/extensions/work/codebook/"
