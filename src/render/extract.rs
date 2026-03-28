use crate::render::{FurGpuBuffers, FurSharedParams};
use crate::Fur;
use bevy::asset::Assets;
use bevy::log::info;
use bevy::mesh::{Indices, Mesh, VertexAttributeValues};
use bevy::prelude::{Commands, Query, Res};
use bevy::render::render_resource::{
    BindGroupEntry, BufferDescriptor, BufferInitDescriptor, BufferUsages,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::sync_world::RenderEntity;
use bevy::render::Extract;
use crate::render::pipeline::FurPipelines;

/// Runs in `ExtractSchedule`: reads `Fur` entities from the main world, builds GPU buffers, and
/// inserts `FurGpuBuffers` as a render-world component. Commands are flushed by
/// `apply_extract_commands` (first step of the render schedule) before `PrepareBindGroups` runs.
pub fn extract_fur_buffers(
    mut commands: Commands,
    fur_query: Extract<Query<(&Fur, &RenderEntity)>>,
    mesh_assets: Extract<Res<Assets<Mesh>>>,
    pipelines: Option<Res<FurPipelines>>,
    shared_params: Option<Res<FurSharedParams>>,
    render_device: Res<RenderDevice>,
    gpu_buffers: Query<Option<&FurGpuBuffers>>,
) {
    let (Some(pipelines), Some(shared_params)) = (pipelines, shared_params) else {
        return;
    };

    for (fur, render_entity) in &fur_query {
        let Some(mesh) = mesh_assets.get(&fur.mesh) else {
            continue;
        };

        // Skip if buffers already exist for the current mode.
        let existing = gpu_buffers.get(render_entity.id()).ok().flatten();
        if existing.is_some_and(|b| b.mode == fur.mode) {
            continue;
        }

        let mode = fur.mode;

        let vertices = mesh_to_vertices(mesh);
        let triangle_count = (vertices.len() / 3) as u32;
        info!("Fur GPU buffers built: {} triangles, mode {:?}", triangle_count, mode);

        let input_buf = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("fur_input"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });
        let output_buf = render_device.create_buffer(&BufferDescriptor {
            label: Some("fur_output"),
            size: triangle_count as u64 * mode.verts_per_tri() as u64 * super::OUTPUT_VERTEX_STRIDE,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let compute_bind_group = render_device.create_bind_group(
            "fur_compute_bg",
            &pipelines.compute_bgl,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: input_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: output_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: shared_params.params_buf.as_entire_binding(),
                },
            ],
        );
        let draw_verts_bind_group = render_device.create_bind_group(
            "fur_draw_verts_bg",
            &pipelines.draw_verts_bgl,
            &[BindGroupEntry {
                binding: 0,
                resource: output_buf.as_entire_binding(),
            }],
        );

        commands.entity(render_entity.id()).insert(FurGpuBuffers {
            input_buf,
            output_buf,
            triangle_count,
            mode,
            compute_bind_group,
            draw_verts_bind_group,
            draw_view_bind_group: None,
        });
    }
}

fn mesh_to_vertices(mesh: &Mesh) -> Vec<FurVertex> {
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(p)) => p,
        _ => panic!("mesh missing Float32x3 positions"),
    };
    let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(n)) => n,
        _ => panic!("mesh missing Float32x3 normals"),
    };
    let vert = |i: usize| {
        let p = positions[i];
        let n = normals[i];
        FurVertex {
            position: [p[0], p[1], p[2], 0.0],
            normal: [n[0], n[1], n[2], 0.0],
        }
    };
    let mut verts = Vec::new();
    match mesh.indices() {
        Some(Indices::U32(idx)) => {
            for c in idx.chunks_exact(3) {
                verts.extend([
                    vert(c[0] as usize),
                    vert(c[1] as usize),
                    vert(c[2] as usize),
                ]);
            }
        }
        Some(Indices::U16(idx)) => {
            for c in idx.chunks_exact(3) {
                verts.extend([
                    vert(c[0] as usize),
                    vert(c[1] as usize),
                    vert(c[2] as usize),
                ]);
            }
        }
        None => {
            for i in (0..positions.len()).step_by(3) {
                verts.extend([vert(i), vert(i + 1), vert(i + 2)]);
            }
        }
    }
    verts
}

/// GPU-ready vertex — 32 bytes, matches `InputVertex` in the compute shaders.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct FurVertex {
    position: [f32; 4],
    normal: [f32; 4],
}