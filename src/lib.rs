pub mod texture;
pub mod buffer;
pub mod gpu;
pub mod bindgroup;
pub mod resource;

pub mod prelude {
    pub use super::{bindgroup::*, buffer::*, gpu::*, resource::*, texture::*};
    pub use wgpu;
    pub use winit;
    pub use glam;
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsError;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

use winit::{
    dpi::PhysicalSize, window::Window, event_loop::ActiveEventLoop
};

// use std::time::SystemTime;

#[cfg(target_arch = "wasm32")]
use pollster::FutureExt;

#[cfg(target_arch = "wasm32")]
use wgpu::util::DeviceExt;



///////////////////////////////////////////////////////////////////////////////
/*
This file contains the GPU struct and wrapper structs for some WGPU resources
(and a handful of helper functions for windowing)

The wrappers were a bad idea, I thought I would be using wgpu more
and started on helper structs before I actually had a use for them
*/
///////////////////////////////////////////////////////////////////////////////


#[cfg(not(target_arch = "wasm32"))]
pub fn new_window(event_loop: &ActiveEventLoop, res: [u32; 2]) -> Result<winit::window::Window, winit::error::OsError> {
    event_loop.create_window(Window::default_attributes().with_inner_size(PhysicalSize::new(res[0], res[1])))
}

#[cfg(target_arch = "wasm32")]
pub fn new_window_in_canvas(event_loop: &ActiveEventLoop, canvas_id: &'static str) -> Result<winit::window::Window, JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    use wasm_bindgen::JsCast;
    use winit::platform::web::WindowBuilderExtWebSys;

    let canvas = web_sys::window()
        .ok_or_else(|| JsValue::from_str("Could not get window object"))?
        .document()
        .ok_or_else(|| JsValue::from_str("Could not get document object"))?
        .get_element_by_id(canvas_id)
        .ok_or_else(|| JsValue::from_str(&format!("Could not find canvas element with id '{}'", canvas_id)))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|e| JsValue::from_str(&format!("Could not cast element to HtmlCanvasElement: {:?}", e)))?;

    let window = event_loop.create_window(Window::default_attributes().with_canvas(Some(canvas)))
        .map_err(|e| JsValue::from_str(&format!("Error building window: {:?}", e)))?;

    Ok(window)
}


// pub struct Shader {
//     path: String,
//     deps: Vec<usize>,
//     compiled: SystemTime,
//     module: wgpu::ShaderModule,
// }

// impl Shader {
//     pub fn new() -> Shader {
//         wgpu::ShaderModule::
//     }
// }

/// Fetch the bytes of a file. Returns None if an error occurred
/// 
/// # Panics
/// when targeting WASM, panics if the file path is not found
pub async fn fetch_bytes(path: &str) -> Option<Vec<u8>> {
    #[cfg(not(target_arch = "wasm32"))] 
    {
        if let Ok(bytes) = std::fs::read(path) {
            Some(bytes)
        } else {
            None
        }

    }
    
    #[cfg(target_arch = "wasm32")] 
    {
        let Ok(js_future) = JsFuture::from(web_sys::window()?.fetch_with_str(path)).await 
            else {return None};

        let Ok(response) = js_future.dyn_into::<Response>()
            else {return None};

        let Ok(array_buf) = response.array_buffer()
            else {return None};

        let Ok(array_buf) = JsFuture::from(array_buf).await 
            else {return None};

        let typed_arr = js_sys::Uint8Array::new(&array_buf);

        Some(typed_arr.to_vec())
    }
}

