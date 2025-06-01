[working-directory: "lsb-js"]
build-js:
  RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack build --release

test:
  #!/usr/bin/env bash
  cargo test --release
  cd lsb-py
  maturin develop --release
  python -m unittest discover
  cd ../lsb-js
  RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack test --node --release
