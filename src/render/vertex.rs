use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use bytemuck::{Pod, Zeroable};

pub trait VertexData<'l>: Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {
    const SIZE: usize;
    const ATTRIBUTES: &'l [wgpu::VertexAttribute];
    const LAYOUT: wgpu::VertexBufferLayout<'l>;
}

pub type IndexValue = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Index(pub IndexValue);

impl Index {
    pub const FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint32;
}

#[derive(Debug, Default, Clone)]
pub struct IndexList(pub Vec<Index>);

impl IndexList {
    pub fn new() -> Self {
        IndexList(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        IndexList(Vec::with_capacity(capacity))
    }

    pub fn push(&mut self, index: Index) {
        self.0.push(index);
    }

    pub fn push_value(&mut self, index: IndexValue) {
        self.0.push(Index(index));
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn create_init_wgpu_buff(&self, d: &wgpu::Device) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;

        d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(self.as_ref()),
            usage: wgpu::BufferUsages::INDEX,
        })
    }
}

impl<T: AsRef<[IndexValue]>> From<T> for IndexList {
    fn from(it: T) -> Self {
        IndexList(it.as_ref().iter().map(|it| Index(*it)).collect())
    }
}

impl Deref for IndexList {
    type Target = [IndexValue];

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self.0.as_slice()) }
    }
}

impl DerefMut for IndexList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let indices: *mut [Index] = self.0.as_mut_slice();
            let target: *mut Self::Target = indices as *mut Self::Target;
            target.as_mut().unwrap_unchecked()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StaticVertexBuffer<D: VertexData<'static>>(pub &'static [D]);

#[derive(Clone)]
pub struct VertexBuffer<'l, D: VertexData<'l>> {
    data: Vec<D>,
    _interface: PhantomData<&'l ()>,
}

impl<'l, D: VertexData<'l>> Default for VertexBuffer<'l, D> {
    fn default() -> Self {
        VertexBuffer {
            data: Vec::new(),
            _interface: PhantomData::default(),
        }
    }
}

impl<'l, D: VertexData<'l>> VertexBuffer<'l, D> {
    pub fn new() -> Self {
        VertexBuffer {
            data: Vec::with_capacity(512),
            _interface: PhantomData::default(),
        }
    }

    pub fn layout() -> wgpu::VertexBufferLayout<'l> {
        D::LAYOUT
    }

    pub fn create_init_wgpu_buff(&self, d: &wgpu::Device) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;

        d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.data),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }
}

impl<D: VertexData<'static>> From<StaticVertexBuffer<D>> for VertexBuffer<'static, D> {
    fn from(static_buffer: StaticVertexBuffer<D>) -> Self {
        VertexBuffer {
            data: Vec::from(static_buffer.0),
            _interface: PhantomData::default(),
        }
    }
}

impl<'l, D: VertexData<'l> + Debug> Debug for VertexBuffer<'l, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VertexBuffer")
            .field("data", &self.data)
            .finish()
    }
}

impl<'l, D: VertexData<'l>> Deref for VertexBuffer<'l, D> {
    type Target = [D];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'l, D: VertexData<'l>> DerefMut for VertexBuffer<'l, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[derive(Debug, Copy, Clone, Pod, Zeroable, VertexData)]
#[repr(C)]
pub struct DevVertexData {
    pub position: [f32; 3],
    pub color: [f32; 3],
}
