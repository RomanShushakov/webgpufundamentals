cd ./fundamentals && \
RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --release --no-default-features --target wasm32-unknown-unknown && \
wasm-bindgen --target web --out-name fundamentals --out-dir ../../wasm --no-typescript ./target/wasm32-unknown-unknown/release/fundamentals.wasm
