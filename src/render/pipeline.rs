use wgpu::*;

use super::shader::ShaderSource;

#[derive(Debug)]
pub struct VertexInterface<'a> {
    pub entry_point: String,
    pub buffers: Vec<VertexBufferLayout<'a>>,
}

#[derive(Debug)]
pub struct FragmentInterface {
    pub entry_point: String,
    pub targets: Vec<Option<ColorTargetState>>,
}

#[derive(Debug)]
pub struct Pipeline<'v, S: ShaderSource> {
    pub shader: S,

    pub vertex_interface: VertexInterface<'v>,
    pub fragment_interface: Option<FragmentInterface>,

    pub topology: PrimitiveTopology,
    pub polygon_mode: PolygonMode,

    shader_module: Option<ShaderModule>,
}

impl<'v, S: ShaderSource> Pipeline<'v, S> {
    pub fn new(
        shader: S,
        vertex_interface: VertexInterface<'v>,
        fragment_interface: Option<FragmentInterface>,
    ) -> Self {
        Pipeline {
            shader,
            vertex_interface,
            fragment_interface,

            topology: PrimitiveTopology::TriangleList,
            polygon_mode: PolygonMode::Fill,

            shader_module: None,
        }
    }

    pub fn create_render_pipeline(&self, device: &Device) -> RenderPipeline {
        let shader = self
            .shader_module
            .get_or_insert_with(|| self.shader.create_shader_module(device));

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: &self.vertex_interface.entry_point,
                buffers: &self.vertex_interface.buffers,
            },
            fragment: self.fragment_interface.map(|interface| FragmentState {
                module: &shader,
                entry_point: &interface.entry_point,
                targets: &interface.targets.as_slice(),
            }),
            primitive: PrimitiveState {
                topology: self.topology,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                // Requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: self.polygon_mode,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }
}
