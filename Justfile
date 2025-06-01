[working-directory: "lsb-js"]
build-js:
  RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack build --release


