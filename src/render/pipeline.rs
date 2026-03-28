use crate::render::FurSharedParams;
use bevy::asset::AssetServer;
use bevy::core_pipeline::core_3d::CORE_3D_DEPTH_FORMAT;
use bevy::image::BevyDefault;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::{default, Commands, Res, ResMut, Resource};
use bevy::render::render_resource::{BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, BufferDescriptor, BufferUsages, CachedComputePipelineId, CachedRenderPipelineId, ColorTargetState, ColorWrites, CompareFunction, ComputePipelineDescriptor, DepthStencilState, FragmentState, MultisampleState, PipelineCache, PrimitiveState, RenderPipelineDescriptor, ShaderStages, ShaderType, TextureFormat, VertexState};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::view::ViewUniform;
use crate::FurMode;

pub fn setup_fur_pipelines(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: ResMut<PipelineCache>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let compute_bgl_desc = BindGroupLayoutDescriptor::new(
        "fur_compute_bgl",
        &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    );

    let draw_verts_bgl_desc = BindGroupLayoutDescriptor::new(
        "fur_draw_verts_bgl",
        &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    );

    let draw_view_bgl_desc = BindGroupLayoutDescriptor::new(
        "fur_draw_view_bgl",
        &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: Some(ViewUniform::min_size()),
            },
            count: None,
        }],
    );

    let compute_bgl = pipeline_cache.get_bind_group_layout(&compute_bgl_desc);
    let draw_verts_bgl = pipeline_cache.get_bind_group_layout(&draw_verts_bgl_desc);
    let draw_view_bgl = pipeline_cache.get_bind_group_layout(&draw_view_bgl_desc);

    let draw_shader =
        asset_server.load("embedded://bevy_fur/shaders/fur_draw.wgsl");

    let compute_ids = [
        asset_server.load("embedded://bevy_fur/shaders/fur_compute_1.wgsl"),
        asset_server.load("embedded://bevy_fur/shaders/fur_compute_2.wgsl"),
        asset_server.load("embedded://bevy_fur/shaders/fur_compute_3.wgsl"),
        asset_server.load("embedded://bevy_fur/shaders/fur_compute_4.wgsl"),
    ]
    .map(|shader| {
        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("fur_compute_pipeline".into()),
            layout: vec![compute_bgl_desc.clone()],
            push_constant_ranges: vec![],
            shader,
            shader_defs: vec![],
            entry_point: Some("main".into()),
            zero_initialize_workgroup_memory: false,
        })
    });

    let draw_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("fur_draw_pipeline".into()),
        layout: vec![draw_verts_bgl_desc, draw_view_bgl_desc],
        push_constant_ranges: vec![],
        vertex: VertexState {
            shader: draw_shader.clone(),
            entry_point: Some("vs_main".into()),
            buffers: vec![],
            shader_defs: vec![],
        },
        fragment: Some(FragmentState {
            shader: draw_shader,
            entry_point: Some("fs_main".into()),
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
            shader_defs: vec![],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            cull_mode: None,
            ..default()
        },
        depth_stencil: Some(DepthStencilState {
            format: CORE_3D_DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: CompareFunction::GreaterEqual,
            stencil: default(),
            bias: default(),
        }),
        multisample: MultisampleState {
            count: 4,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        zero_initialize_workgroup_memory: false,
    });

    let params_buf = render_device.create_buffer(&BufferDescriptor {
        label: Some("fur_params"),
        size: super::PARAMS_BUF_SIZE as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    render_queue.write_buffer(&params_buf, 0, &[0u8; super::PARAMS_BUF_SIZE]);

    commands.insert_resource(FurSharedParams { params_buf });
    commands.insert_resource(FurPipelines {
        compute_ids,
        draw_id,
        compute_bgl,
        draw_verts_bgl,
        draw_view_bgl,
    });
}

#[derive(Resource)]
pub struct FurPipelines {
    pub compute_ids: [CachedComputePipelineId; 4],
    pub draw_id: CachedRenderPipelineId,
    pub compute_bgl: BindGroupLayout,
    pub draw_verts_bgl: BindGroupLayout,
    pub draw_view_bgl: BindGroupLayout,
}

impl FurPipelines {
    pub fn compute_id(&self, mode: &FurMode) -> CachedComputePipelineId {
        match mode {
            FurMode::Approach1 => self.compute_ids[0],
            FurMode::Approach2 => self.compute_ids[1],
            FurMode::Approach3 => self.compute_ids[2],
            FurMode::Approach4 => self.compute_ids[3],
        }
    }
}