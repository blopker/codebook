MAKEFLAGS += -j4
.PHONY: *

test:
	cargo test

build:
	cargo build
