use std::collections::HashMap;

use crate::bindgroup::BindGroupLayoutEntries;


/// Caches bind group layouts in probably the least efficient way possible
/// 
/// Not sure why I made this, thought I would be recreating bind groups a lot more often
#[derive(Default)]
pub struct ResourceManager {
    pub bind_group_layouts: HashMap<BindGroupLayoutEntries, wgpu::BindGroupLayout>,
    // shaders: Vec<Shader>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_bind_group_layout(&self, layout_entries: &BindGroupLayoutEntries) -> Option<&wgpu::BindGroupLayout> {
        self.bind_group_layouts.get(layout_entries)
    }

    // async fn get_shader(&mut self, path: String) -> Option<usize> {
    //     if let Some(i) = self.shaders.iter().position(|s| { s.path == path }) {
    //         return Some(i);
    //     }

    //     let Some(bytes) = fetch_bytes(&path).await else {
    //         return None;
    //     };
    //     let Ok(contents) = str::from_utf8(bytes.as_slice()) else {
    //         return None;
    //     };

    //     for line in contents.lines() {
    //         if let Some(rel) = line.trim().strip_prefix("@import ").and_then(|s| {
    //             s.trim().strip_prefix('"')?.strip_suffix('"')
    //         }) {
                
    //         }
    //     }

    //     Some(self.shaders.len() - 1)
    // }

    // pub async fn add_shader(path: String) -> bool {

        

    //     true
    // }

}