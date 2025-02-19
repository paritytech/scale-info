.PHONY: check test clean

check:
	cargo clippy --all-targets -- -D warnings
	cargo clippy --all --all-features -- -D warnings
	cargo check --no-default-features --features bit-vec
	cargo check --no-default-features --features docs
	cargo check --no-default-features --features serde
	cargo check --no-default-features --features serde,decode
	cargo check --no-default-features --features schema

test:
	cargo test --all --all-features
	cd ./test_suite/derive_tests_no_std; \
		cargo run --target x86_64-unknown-none --no-default-features

clean:
	cargo clean
	cd ./test_suite/derive_tests_no_std; \
		cargo clean
