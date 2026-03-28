use crate::mode::FurMode;
use bevy::{
    core_pipeline::core_3d::{graph::Core3d, graph::Node3d},
    prelude::*,
    render::{
        render_graph::{RenderGraphExt, RenderLabel, ViewNodeRunner},
        render_resource::{
            BindGroup, BindGroupEntry, Buffer
            ,
        },
        renderer::{RenderDevice, RenderQueue},
        view::ViewUniforms,
        ExtractSchedule, RenderApp, RenderStartup, RenderSystems,
    },
};
use compute::FurComputeNode;
use draw::FurDrawNode;
use pipeline::FurPipelines;

mod compute;
mod draw;
mod extract;
mod pipeline;

/// Size of the fur parameters uniform buffer: one `vec4<f32>` (time + 3 padding floats).
const PARAMS_BUF_SIZE: usize = 16;
/// Byte stride of one `OutputVertex` in the compute output buffer (`position: vec4` + `color: vec4`).
const OUTPUT_VERTEX_STRIDE: u64 = 32;

const ALL_MODES: [FurMode; 4] = [
    FurMode::Approach1,
    FurMode::Approach2,
    FurMode::Approach3,
    FurMode::Approach4,
];

// ---- Vertex type -----------------------------------------------

// ---- GPU resources ---------------------------------------------

/// Shared render-world resource holding the per-frame time uniform.
#[derive(Resource)]
struct FurSharedParams {
    params_buf: Buffer,
}

/// Per-entity render-world component holding all GPU buffers for one fur mesh.
#[derive(Component)]
struct FurGpuBuffers {
    #[allow(dead_code)]
    input_buf: Buffer,
    #[allow(dead_code)]
    output_buf: Buffer,
    triangle_count: u32,
    mode: FurMode,
    compute_bind_group: BindGroup,
    draw_verts_bind_group: BindGroup,
    draw_view_bind_group: Option<BindGroup>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
enum FurNode {
    Compute,
    Draw,
}

// ---- Plugin ----------------------------------------------------

pub struct FurRenderPlugin;

impl Plugin for FurRenderPlugin {
    fn build(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .add_systems(RenderStartup, pipeline::setup_fur_pipelines)
            .add_systems(ExtractSchedule, extract::extract_fur_buffers)
            .add_systems(
                bevy::render::Render,
                (
                    update_fur_params.in_set(RenderSystems::PrepareResources),
                    prepare_view_bind_group.in_set(RenderSystems::PrepareBindGroups),
                ),
            )
            .add_render_graph_node::<FurComputeNode>(Core3d, FurNode::Compute)
            .add_render_graph_node::<ViewNodeRunner<FurDrawNode>>(Core3d, FurNode::Draw)
            .add_render_graph_edge(Core3d, Node3d::StartMainPass, FurNode::Compute)
            .add_render_graph_edge(Core3d, FurNode::Compute, FurNode::Draw)
            .add_render_graph_edge(Core3d, FurNode::Draw, Node3d::MainOpaquePass);
    }
}

// ---- Render systems --------------------------------------------

fn update_fur_params(
    time: Res<Time>,
    shared_params: Option<Res<FurSharedParams>>,
    render_queue: Res<RenderQueue>,
) {
    let Some(shared_params) = shared_params else {
        return;
    };
    let mut data = [0u8; PARAMS_BUF_SIZE];
    data[0..4].copy_from_slice(&time.elapsed_secs().to_le_bytes());
    render_queue.write_buffer(&shared_params.params_buf, 0, &data);
}

fn prepare_view_bind_group(
    mut query: Query<&mut FurGpuBuffers>,
    pipelines: Option<Res<FurPipelines>>,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
) {
    let Some(pipelines) = pipelines else {
        return;
    };
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };
    for mut buffers in &mut query {
        buffers.draw_view_bind_group = Some(render_device.create_bind_group(
            "fur_draw_view_bg",
            &pipelines.draw_view_bgl,
            &[BindGroupEntry {
                binding: 0,
                resource: view_binding.clone(),
            }],
        ));
    }
}
