MAKEFLAGS += -j4
.PHONY: *
export RUST_LOG=info

test:
	cargo test --lib --bins --tests -- --test-threads=20

build: generate_word_list
	cd crates/codebook-lsp && cargo build

build-release: generate_word_list
	cd crates/codebook-lsp && cargo build --release

integration_test: build
	cd integration_tests && bun test

install_ext: build-release
	cp -f target/release/codebook-lsp "${HOME}/Library/Application Support/Zed/extensions/work/codebook/"

uninstall_ext:
	rm -f "${HOME}/Library/Application Support/Zed/extensions/work/codebook/codebook-lsp"

generate_word_list:
	bun run scripts/generate_combined_wordlist.ts

release-lsp:
	bun run scripts/release_lsp.ts
