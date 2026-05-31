test-all:
  cargo test
  cargo test --features word-i32

examples:
  cargo run --example disassembly

lint:
  cargo clippy

all: lint test-all

fuzz-f32:
  cargo fuzz run fuzz_target_f32
