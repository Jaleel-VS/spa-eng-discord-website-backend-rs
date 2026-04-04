.PHONY: test dev

test:
	cargo test -- --test-threads=1

dev:
	cargo run
