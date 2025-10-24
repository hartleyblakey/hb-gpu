use std::fmt;
use std::io;
use image::ImageError;

pub struct Texture {
    pub label: Option<String>,
    pub dim: wgpu::TextureViewDimension,
    pub raw: wgpu::Texture,
    pub views: Vec<TextureView>,
}

pub struct TextureView {
    pub raw: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub dimension: wgpu::TextureViewDimension,
    pub aspect: wgpu::TextureAspect,
    pub base_mip_level: u32,
    pub mip_level_count: Option<u32>,
    pub base_array_layer: u32,
    pub array_layer_count: Option<u32>,
}


impl Texture {
    pub fn view_all(&self) -> &TextureView {
        &self.views[0]
    }

    pub fn default_label(&self) -> String {    
        format!("{} channel, {} bytes per pixel, {} by {} by {} texture", 
            self.raw.format().components(), 
            self.raw.format().target_pixel_byte_cost().unwrap_or(0), 
            self.raw.size().width, 
            self.raw.size().height,
            self.raw.size().depth_or_array_layers,
        )
    }

    pub fn new_view(&mut self) -> &TextureView {
        let s;
        let label = if self.label.is_some() {
            s = format!("View of {}", self.label.as_ref().unwrap());
            Some(s.as_str())
        } else {
            None
        };

        let mut desc = wgpu::TextureViewDescriptor::default();
        desc.label = label;


        let view = TextureView {
            raw: self.raw.create_view(&desc),
            format: self.raw.format(),
            dimension: wgpu::TextureViewDimension::D2,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        };

        self.views.push(view);
        self.views.last().unwrap()
    }
    
}

impl std::ops::Deref for Texture {
    type Target = wgpu::Texture;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl std::ops::Deref for TextureView {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}


impl TextureView {
    pub fn attachment(&self) -> wgpu::RenderPassColorAttachment {
        wgpu::RenderPassColorAttachment {
            view: &self.raw,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        }
    }
}


#[derive(Debug)]
pub enum TextureError {
    IoError(io::Error),
    ImageError(ImageError),
    Other(String),
}

impl fmt::Display for TextureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextureError::IoError(e) => write!(f, "IO error: {}", e),
            TextureError::ImageError(e) => write!(f, "Image error: {}", e),
            TextureError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl From<io::Error> for TextureError {
    fn from(error: io::Error) -> Self {
        TextureError::IoError(error)
    }
}

impl From<ImageError> for TextureError {
    fn from(error: ImageError) -> Self {
        TextureError::ImageError(error)
    }
}

