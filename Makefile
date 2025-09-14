MAKEFLAGS += -j4
.PHONY: *
export RUST_LOG=debug

test:
	cargo test --lib --bins --tests -- --test-threads=20

test_queries:
	cd crates/codebook && cargo test queries::tests::test_all_queries_are_valid

build:
	cd crates/codebook-lsp && cargo build --release

integration_test: build
	cd integration_tests && bun test

# Build and install 'fast-build' dev version into Zed's extension directory for testing
install_ext: generate_word_list
	cd crates/codebook-lsp && cargo build --profile=fast-release
	cp -f target/fast-release/codebook-lsp "${HOME}/Library/Application Support/Zed/extensions/work/codebook/"

# Install release version into Zed's extension directory for testing
install_release_ext: generate_word_list
	cd crates/codebook-lsp && cargo build --release
	cp -f target/release/codebook-lsp "${HOME}/Library/Application Support/Zed/extensions/work/codebook/"

uninstall_ext:
	rm -f "${HOME}/Library/Application Support/Zed/extensions/work/codebook/codebook-lsp"

generate_word_list:
	bun run scripts/generate_combined_wordlist.ts

release-lsp:
	bun run scripts/release_lsp.ts

clear_cache: build
	target/debug/codebook-lsp clean

benchmark:
	cd crates/codebook && cargo build --release
	./target/release/codebook --benchmark

build-dictionaries:
	cargo run -p dictionary-builder -- build

generate-manifest:
	cargo run -p dictionary-builder -- generate-manifest

publish_crates:
	# 1. First, publish the codebook-config crate
	-cargo publish -p codebook_config
	# 2. Then publish the renamed downloader crate
	-cargo publish -p codebook_downloader
	# 3. Then publish the main codebook library
	-cargo publish -p codebook
	# 4. Finally, publish the codebook-lsp binary
	-cargo publish -p codebook-lsp
