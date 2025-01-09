MAKEFLAGS += -j4
.PHONY: *
export RUST_LOG=info

test:
	cargo test

build:
	cd codebook-lsp && cargo build

integration_test: build
	cd integration_tests && .venv/bin/python -m pytest -s
