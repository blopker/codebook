MAKEFLAGS += -j4
.PHONY: *

test:
	cargo test

get_dictionary:
	curl -o index.aff https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/en/index.aff
	curl -o index.dic https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/en/index.dic

build:
	cargo build
