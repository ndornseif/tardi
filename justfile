test-all:
  @echo "============= Testing with default crate features ============="
  cargo test
  @echo "============= Testing with word-i32 crate feature ============="
  cargo test --features word-i32

examples:
  cargo run --example sqrt_stream

lint:
  cargo clippy

all: lint test-all

fuzz-f32:
  cargo fuzz run fuzz_target_f32
