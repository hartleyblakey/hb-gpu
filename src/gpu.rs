use std::sync::Arc;

use bytemuck::bytes_of;
use glam::UVec2;
use image::GenericImageView;
use winit::window::Window;

use crate::{bindgroup::{BGBuilder, BindGroup}, buffer::Buffer, resource::ResourceManager, texture::{Texture, TextureError, TextureView}};

/// Helper struct to hold the core wgpu resources in one place so they are easier 
/// to construct and pass around
pub struct Gpu {
    pub adapter: wgpu::Adapter, 
    pub device:  wgpu::Device, 
    pub queue:   wgpu::Queue, 
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub window:  Arc<Window>,
}


impl Gpu {
    pub fn new_bind_group<'a>(&'a self) -> BGBuilder<'a> {
        BGBuilder::new(&self.device)
    }

    pub fn new_pipeline_layout(&self, resources: &ResourceManager, bind_groups: &[&BindGroup]) -> wgpu::PipelineLayout {
        let layouts: Vec<&wgpu::BindGroupLayout> = bind_groups.iter()
            .map(|bg| resources.get_bind_group_layout(&bg.entries).unwrap())
            .collect();

        let layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        });
        layout
    }

    pub fn new_texture(&self, size: UVec2, format: wgpu::TextureFormat, renderable: bool) -> Texture {
        let usage = if renderable {
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
        } else {
            wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING
        };
        
        let desc = wgpu::TextureDescriptor {
            dimension: wgpu::TextureDimension::D2,
            format,
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            usage,
            view_formats: &[],
        };

        let mut tex = Texture {
            raw: self.device.create_texture(&desc),
            label: None,
            dim: wgpu::TextureViewDimension::D2,
            views: Vec::new(),
        };

        tex.label = Some(tex.default_label());
        tex.new_view();
        tex
    }

    pub fn get_surface_view(&self, surface_texture: &wgpu::SurfaceTexture) -> TextureView {
        let mut surface_view_desc = wgpu::TextureViewDescriptor::default();
        surface_view_desc.format =  Some(self.surface_config.view_formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(self.surface_config.format));
        let view = surface_texture.texture.create_view(&surface_view_desc);
        TextureView {
            raw: view,
            format: surface_view_desc.format.unwrap(),
            dimension: surface_view_desc.dimension.unwrap_or(wgpu::TextureViewDimension::D2),
            aspect: surface_view_desc.aspect,
            base_mip_level: surface_view_desc.base_mip_level,
            mip_level_count: surface_view_desc.mip_level_count,
            base_array_layer: surface_view_desc.base_array_layer,
            array_layer_count: surface_view_desc.array_layer_count,
        }
    }

    pub fn new_texture_from_file(&self, path: &str) -> Result<Texture, TextureError> {
        let reader = image::ImageReader::open(path)?.with_guessed_format()?;
        let image = reader.decode()?;
        let dim = image.dimensions();
        let (format, bytes_per_pixel) = match image {
            image::DynamicImage::ImageLuma8(_)      => (wgpu::TextureFormat::R8Unorm, 1),
            image::DynamicImage::ImageLumaA8(_)     => (wgpu::TextureFormat::Rg8Unorm, 2),
            image::DynamicImage::ImageRgb8(_)       => (wgpu::TextureFormat::Rgba8UnormSrgb, 4),
            image::DynamicImage::ImageRgba8(_)      => (wgpu::TextureFormat::Rgba8UnormSrgb, 4),
            image::DynamicImage::ImageLuma16(_)     => (wgpu::TextureFormat::R32Float, 4),
            image::DynamicImage::ImageLumaA16(_)    => (wgpu::TextureFormat::Rg16Float, 4),
            image::DynamicImage::ImageRgb16(_)      => (wgpu::TextureFormat::Rgba16Float, 8),
            image::DynamicImage::ImageRgba16(_)     => (wgpu::TextureFormat::Rgba16Float, 8),
            image::DynamicImage::ImageRgb32F(_)     => (wgpu::TextureFormat::Rgba32Float, 16),
            image::DynamicImage::ImageRgba32F(_)    => (wgpu::TextureFormat::Rgba32Float, 16),
            _ => (wgpu::TextureFormat::Rgba32Float, 16),
        };
        let tex = self.new_texture(dim.into(), format, false);
        self.queue.write_texture(
            tex.raw.as_image_copy(), 
            image.as_bytes(), 
            wgpu::TexelCopyBufferLayout {
                bytes_per_row: Some(dim.0 * bytes_per_pixel),
                rows_per_image: None,
                offset: 0,
            }, 
            wgpu::Extent3d {
                width: dim.0,
                height: dim.1,
                depth_or_array_layers: 1,
            }
        );
        Ok(tex)
    }


    pub fn new_uniform_buffer<T: bytemuck::Pod>(&self, val: &T) -> Buffer {
        let size = size_of::<T>() as u64;
        let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        let desc = wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size,
            usage
        };
        let buffer = self.device.create_buffer(&desc);
        self.queue.write_buffer(&buffer, 0, bytes_of(val));
        Buffer {
            raw: buffer,
        }
    }

    pub fn new_storage_buffer(&self, size: u64) -> Buffer {
        let usage = 
            wgpu::BufferUsages::STORAGE 
            | wgpu::BufferUsages::COPY_DST 
            | wgpu::BufferUsages::COPY_SRC;

        let desc = wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size,
            usage
        };

        Buffer {
            raw: self.device.create_buffer(&desc),
        }
    }


    pub async fn new(window: Arc<Window>) -> Option<Self> {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window.clone()).ok()?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                    // integrated gpu spammed console with DX12 errors, easy fix

                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");
        
        let mut limits = wgpu::Limits::default();
        limits = limits.using_resolution(adapter.limits());
        
        // request max size buffers
        limits.max_buffer_size = adapter.limits().max_buffer_size;
        limits.max_storage_buffer_binding_size = adapter.limits().max_storage_buffer_binding_size;

        let device_desc = wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
            required_limits: limits,
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            trace: wgpu::Trace::Off,
        };

        // Create the logical device and command queue
        let (device, queue) = adapter.request_device(&device_desc)
            .await
            .expect("Failed to create device");


        device.on_uncaptured_error(Box::new(
            |error| 
            {
                match &error {
                    wgpu::Error::Validation { source: _, description } => {

                        println!("{description}");
                        if description.contains("Device::create_shader_module") {
                            return;
                        }
                    },
                    _ => (),
                }
                println!("Panicking due to uncaptured wgpu error");
                panic!();
            }
        
        ));

        let mut surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        let surface_caps = surface.get_capabilities(&adapter);

        // from https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
        // get first srgb format, or the default format if no srgb formats are supported
        surface_config.format = surface_caps.formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(surface_caps.formats[0]);
        if !surface_config.format.is_srgb() {
            surface_config.view_formats.push(surface_config.format.add_srgb_suffix());
        }   
        surface_config.present_mode = wgpu::PresentMode::AutoNoVsync;

        surface.configure(&device, &surface_config);

        Some(Self {
            adapter,
            device,
            queue,
            surface,
            surface_config,
            window,
        })
    }
}


