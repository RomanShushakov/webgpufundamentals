use wasm_bindgen::prelude::wasm_bindgen;


#[wasm_bindgen]
extern "C"
{
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(value: &str);
}


#[wasm_bindgen]
pub struct Scene {}


#[wasm_bindgen]
impl Scene
{
    pub fn create() -> Self
    {
        Scene {}
    }


    pub fn greeting(&self, input: &str)
    {
        log(&format!("Hello, {input}!"));
    }
}
