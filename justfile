test-all:
  cargo test
  cargo test --features int-word
  cargo test --features long-word
  cargo test --features int-word,long-word
  cargo test --features long-address
  cargo test --features long-instruction

test-all-combinations:
  cargo test
  cargo test --features int-word
  cargo test --features long-word
  cargo test --features int-word,long-word
  cargo test --features long-address
  cargo test --features long-address,int-word
  cargo test --features long-address,long-word
  cargo test --features long-address,int-word,long-word
  cargo test --features long-instruction
  cargo test --features long-instruction,int-word
  cargo test --features long-instruction,long-word
  cargo test --features long-instruction,int-word,long-word
  cargo test --features long-instruction,long-address
  cargo test --features long-instruction,long-address,int-word
  cargo test --features long-instruction,long-address,long-word
  cargo test --features long-instruction,long-address,int-word,long-word

examples:
  cargo run --example sqrt_stream

lint:
  cargo clippy
  cargo clippy --features int-word

all: test-all lint

fuzz-f32:
  cargo fuzz run fuzz_target_f32

fuzz-i32:
  cargo fuzz run --features int-word fuzz_target_i32

