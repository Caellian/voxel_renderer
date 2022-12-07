pub mod pipeline;
pub mod shader;
pub mod texture;
pub mod vertex;

use wgpu::*;
use winit::{event::WindowEvent, window::Window};

use crate::render::{
    vertex::{DevVertexData, StaticVertexBuffer},
};

use vertex::VertexBuffer;

use self::{
    texture::MENU_ICONS,
    vertex::{Index, IndexList, VertexData}, pipeline::{Pipeline, VertexInterface, FragmentInterface}, shader::{WgslSource, DEV_SHADER},
};

pub struct StateConfig {
    present_mode: PresentMode,
}

pub static DEV_VERTICES: StaticVertexBuffer<DevVertexData> = StaticVertexBuffer(&[
    DevVertexData {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    DevVertexData {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    DevVertexData {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
]);

pub struct RendererState {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub surface_config: SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    pub renderer: Renderer<'static>,
}

impl RendererState {
    pub async fn new(window: &Window) -> Self {
        let size = {
            let mut w = window.inner_size();
            w.width = w.width.max(1);
            w.height = w.width.max(1);
            w
        };

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        Limits::downlevel_webgl2_defaults()
                    } else {
                        Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &surface_config);

        let pipeline = Pipeline::new(
            DEV_SHADER.clone(),
            VertexInterface {
                entry_point: "vs_main".to_string(),
                buffers: vec![DevVertexData::LAYOUT],
            },
            Some(FragmentInterface {
                entry_point: "fs_main".to_string(),
                targets: vec![Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            })
        );

        let frame_state = Renderer::new(
            pipeline,
            DEV_VERTICES.into(),
            IndexList::from(&[0, 1, 4, 1, 2, 4, 2, 3, 4]),
            1,
        );

        RendererState {
            surface,
            device,
            queue,
            surface_config,
            size,
            renderer: frame_state,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;

        if !self.renderer.is_configured() {
            self.renderer.configure(&self);
        }

        let out_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.renderer.draw(&mut encoder, &out_view);

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[derive(Debug)]
pub struct Renderer<'v> {
    pub pipeline: Pipeline<'v, WgslSource<'static>>,
    pub vertices: VertexBuffer<'static, DevVertexData>,
    pub index_list: IndexList,
    pub instance_count: u32,

    menu_icons: texture::TextureResource,

    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    render_pipeline: Option<RenderPipeline>,

    menu_icon_bind_group: Option<BindGroup>,
}

impl<'v> Renderer<'v> {
    pub fn new(
        pipeline: Pipeline<'v, WgslSource<'static>>,
        vertices: VertexBuffer<'static, DevVertexData>,
        index_list: IndexList,
        instance_count: u32,
    ) -> Self {
        Renderer {
            pipeline,
            vertices,
            index_list,
            instance_count,

            menu_icons: texture::TextureResource::rgba8_from_memory(MENU_ICONS),

            vertex_buffer: None,
            index_buffer: None,
            render_pipeline: None,
            menu_icon_bind_group: None,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.vertex_buffer.is_some()
            && self.index_buffer.is_some()
            && self.render_pipeline.is_some()
    }

    pub fn configure(&mut self, state: &RendererState) {
        self.vertex_buffer = Some(self.vertices.create_init_wgpu_buff(&state.device));
        self.index_buffer = Some(self.index_list.create_init_wgpu_buff(&state.device));
        self.render_pipeline = Some(
            self.pipeline
                .create_render_pipeline(&state),
        );

        let tex = self
            .menu_icons
            .create_texture_and_upload(&state.device, &state.queue);
        let view = tex.create_view(&TextureViewDescriptor::default());
        let sampler = state.device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("menu_icons_bind_group_layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                multisampled: false,
                                view_dimension: TextureViewDimension::D2,
                                sample_type: TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::FRAGMENT,
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        self.menu_icon_bind_group = Some(bind_group);
    }

    pub(crate) fn draw(&self, commands: &mut CommandEncoder, output: &wgpu::TextureView) {
        let mut render_pass = commands.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        unsafe {
            render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap_unchecked());
            render_pass
                .set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap_unchecked().slice(..));
            render_pass.set_index_buffer(
                self.index_buffer.as_ref().unwrap_unchecked().slice(..),
                Index::FORMAT,
            );
        }

        render_pass.draw(0..self.vertices.len() as u32, 0..self.instance_count as u32);
    }
}
