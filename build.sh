wasmModules=(
    fundamentals
    inter_stage_variables
    uniforms
    storage_buffers
    vertex_buffers
    textures
    loading_images
)

cd ./wasm_modules
cargo build --workspace --release --no-default-features --target wasm32-unknown-unknown

len=(${#wasmModules[@]})

for((i=0; i<len; i+=1));
do 
    wasm-bindgen \
        --target web --out-name ${wasmModules[i]} \
        --out-dir ../wasm --no-typescript \
        ./target/wasm32-unknown-unknown/release/${wasmModules[i]}.wasm
done
