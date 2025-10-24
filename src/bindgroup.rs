use crate::resource::*;
use crate::buffer::*;
use crate::texture::*;

pub struct BGBuilder<'a> {
    layout_entries: BindGroupLayoutEntries,
    entries:        Vec<wgpu::BindGroupEntry<'a>>,
    device:         &'a wgpu::Device,
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct BindGroupLayoutEntries {
    entries: Vec<wgpu::BindGroupLayoutEntry>
}

pub struct BindGroup {
    pub raw: wgpu::BindGroup,
    pub entries: BindGroupLayoutEntries,
}


impl<'a> BGBuilder<'a> {
    pub fn new(device: &wgpu::Device) -> BGBuilder {
        BGBuilder {
            device: &device,
            entries: Vec::new(),
            layout_entries: BindGroupLayoutEntries{entries: Vec::new()},
        }
    }

    pub fn with_buffer(&mut self, view: &'a BufferView, visibility: wgpu::ShaderStages) -> &mut Self {
        let ty : wgpu::BufferBindingType = match view.buffer.raw.usage() {
            _ if view.buffer.raw.usage().contains(wgpu::BufferUsages::UNIFORM)  => wgpu::BufferBindingType::Uniform,
            _ if view.buffer.raw.usage().contains(wgpu::BufferUsages::STORAGE)  => wgpu::BufferBindingType::Storage { read_only: false },
            _ => panic!("Invalid buffer usage: expected uniform or storage"),
        };

        let layout_entry = wgpu::BindGroupLayoutEntry {
            binding: self.layout_entries.entries.len() as u32,
            count: None,
            visibility,
            ty: wgpu::BindingType::Buffer { ty, has_dynamic_offset: false, min_binding_size: None }
        };

        self.layout_entries.entries.push(layout_entry);

        let entry = wgpu::BindGroupEntry {
            binding: self.entries.len() as u32,
            resource: wgpu::BindingResource::Buffer(view.binding()),
        };
        self.entries.push(entry);
        self
    }

    pub fn with_texture(&mut self, texture: &'a Texture, visibility: wgpu::ShaderStages) -> &mut Self {
        
        let ty = wgpu::BindingType::Texture {
            sample_type: texture.raw.format().sample_type(None, None).unwrap_or_default(), 
            view_dimension: texture.dim, 
            multisampled: false // no multisampling for now
        };

        let layout_entry = wgpu::BindGroupLayoutEntry {
            binding: self.layout_entries.entries.len() as u32,
            count: None,
            visibility,
            ty
        };

        self.layout_entries.entries.push(layout_entry);

        let entry = wgpu::BindGroupEntry {
            binding: self.entries.len() as u32,
            resource: wgpu::BindingResource::TextureView(&texture.view_all().raw)
        };

        self.entries.push(entry);
        self
    }

    pub fn finish<'b>(&mut self, manager: &'b mut ResourceManager) -> BindGroup {
        if !manager.bind_group_layouts.contains_key(&self.layout_entries) {
            println!("Created new bind group layout");
            let layout_desc = wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: self.layout_entries.entries.as_slice(),
            };
            manager.bind_group_layouts.insert(self.layout_entries.clone(), self.device.create_bind_group_layout(&layout_desc));
        }

        let layout = manager.get_bind_group_layout(&self.layout_entries).expect("hash get failed after insertion");

        let desc = wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: self.entries.as_slice(),
        };

        BindGroup {
            raw: self.device.create_bind_group(&desc),
            entries: self.layout_entries.clone()
        }
    }
}
