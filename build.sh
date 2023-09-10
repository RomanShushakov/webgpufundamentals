cd ./wasm_modules/fundamentals && \
cargo build --release --no-default-features --target wasm32-unknown-unknown && \
wasm-bindgen --target web --out-name fundamentals --out-dir ../../wasm --no-typescript ./target/wasm32-unknown-unknown/release/fundamentals.wasm && \
cd ../inter_stage_variables && \
cargo build --release --no-default-features --target wasm32-unknown-unknown && \
wasm-bindgen --target web --out-name inter_stage_variables --out-dir ../../wasm --no-typescript ./target/wasm32-unknown-unknown/release/inter_stage_variables.wasm && \
cd ../uniforms && \
cargo build --release --no-default-features --target wasm32-unknown-unknown && \
wasm-bindgen --target web --out-name uniforms --out-dir ../../wasm --no-typescript ./target/wasm32-unknown-unknown/release/uniforms.wasm && \
cd ../storage_buffers && \
cargo build --release --no-default-features --target wasm32-unknown-unknown && \
wasm-bindgen --target web --out-name storage_buffers --out-dir ../../wasm --no-typescript ./target/wasm32-unknown-unknown/release/storage_buffers.wasm
