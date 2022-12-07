use wgpu::{Device, ShaderModule};

use crate::util::CowStr;

pub trait ShaderSource {
    fn create_shader_module(&self, device: &wgpu::Device) -> wgpu::ShaderModule;
}

#[derive(Debug, Clone)]
pub struct WgslSource<'a> {
    pub source: CowStr<'a>,
}

impl WgslSource<'static> {
    pub const fn new_static(source: &'static str) -> Self {
        WgslSource {
            source: source.into(),
        }
    }
}

impl<'a> WgslSource<'a> {
    pub fn new(source: impl Into<CowStr<'a>>) -> Self {
        WgslSource {
            source: source.into(),
        }
    }
}

impl<'a> ShaderSource for WgslSource<'a> {
    fn create_shader_module(&self, device: &Device) -> ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(self.source.into()),
        })
    }
}

pub static DEV_SHADER: WgslSource<'static> = WgslSource::new_static(include_str!("shader.wgsl"));
