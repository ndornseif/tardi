test-all:
  cargo test
  cargo test --features int-word
  cargo test --features long-word
  cargo test --features int-word,long-word
  cargo test --features long-address
  cargo test --features long-instruction

examples:
  cargo run --example sqrt_stream

lint:
  cargo clippy

all: test-all lint

fuzz-f32:
  cargo fuzz run fuzz_target_f32

fuzz-i32:
  cargo fuzz run --features int-word fuzz_target_i32
