wasmModules=(
    ./wasm_modules/fundamentals fundamentals
    ../inter_stage_variables inter_stage_variables
    ../uniforms uniforms
    ../storage_buffers storage_buffers
    ../vertex_buffers vertex_buffers
    ../textures textures 
)

len=(${#wasmModules[@]})
step=2

for((i=0; i<len; i+=step));
do 
    cd ${wasmModules[i]} && \
    cargo build --release --no-default-features --target wasm32-unknown-unknown && \
    wasm-bindgen --target web --out-name ${wasmModules[i+1]} --out-dir ../../wasm --no-typescript ./target/wasm32-unknown-unknown/release/${wasmModules[i+1]}.wasm
done
