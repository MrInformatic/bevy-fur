use crate::render::{FurGpuBuffers, ALL_MODES};
use bevy::prelude::{FromWorld, QueryState, World};
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext};
use bevy::render::render_resource::{ComputePassDescriptor, PipelineCache};
use bevy::render::renderer::RenderContext;
use crate::render::pipeline::FurPipelines;

pub struct FurComputeNode {
    query: QueryState<&'static FurGpuBuffers>,
}

impl FromWorld for FurComputeNode {
    fn from_world(world: &mut World) -> Self {
        FurComputeNode {
            query: world.query(),
        }
    }
}

impl Node for FurComputeNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let Some(pipelines) = world.get_resource::<FurPipelines>() else {
            return Ok(());
        };
        let pipeline_cache = world.resource::<PipelineCache>();
        let all_buffers: Vec<_> = self.query.iter_manual(world).collect();

        let encoder = render_context.command_encoder();
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("fur_compute"),
            timestamp_writes: None,
        });

        // Set pipeline once per mode to avoid redundant pipeline switches.
        for mode in ALL_MODES {
            let mode_buffers: Vec<_> = all_buffers.iter().filter(|b| b.mode == mode).collect();
            if mode_buffers.is_empty() {
                continue;
            }
            let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipelines.compute_id(&mode))
            else {
                continue;
            };
            pass.set_pipeline(pipeline);
            for buffers in mode_buffers {
                pass.set_bind_group(0, &buffers.compute_bind_group, &[]);
                pass.dispatch_workgroups(buffers.triangle_count, 1, 1);
            }
        }
        Ok(())
    }
}
