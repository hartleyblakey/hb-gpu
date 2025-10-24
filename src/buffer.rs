use std::num::NonZero;



impl std::ops::Deref for Buffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

pub struct Buffer {
    pub raw: wgpu::Buffer,
}

impl Buffer {
    pub fn view<'a>(&'a self, offset: u64, size: u64) -> BufferView<'a> {
        BufferView {
            buffer: self,
            offset,
            size,
            read_only: false
        }
    }

    pub fn view_all<'a>(&'a self) -> BufferView<'a> {
        BufferView {
            buffer: self,
            offset: 0,
            size: self.raw.size(),
            read_only: false
        }
    }

    pub fn view_read<'a>(&'a self, offset: u64, size: u64) -> BufferView<'a> {
        BufferView {
            buffer: self,
            offset,
            size,
            read_only: true
        }
    }
}

pub struct BufferView<'a> {
    pub buffer: &'a Buffer,
    pub offset: u64,
    pub size: u64,
    pub read_only: bool
}

impl<'a> BufferView<'a> {
    pub fn binding(&self) -> wgpu::BufferBinding<'a> {
        wgpu::BufferBinding {
            buffer: &self.buffer.raw,
            offset: self.offset,
            size: NonZero::<u64>::new(self.size),
        }
    }
}

