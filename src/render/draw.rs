use crate::render::FurGpuBuffers;
use bevy::ecs::query::QueryItem;
use bevy::prelude::{FromWorld, QueryState, World};
use bevy::render::render_graph::{NodeRunError, RenderGraphContext, ViewNode};
use bevy::render::render_resource::{PipelineCache, RenderPassDescriptor, StoreOp};
use bevy::render::renderer::RenderContext;
use bevy::render::view::{ViewDepthTexture, ViewTarget, ViewUniformOffset};
use crate::render::pipeline::FurPipelines;

pub struct FurDrawNode {
    query: QueryState<&'static FurGpuBuffers>,
}

impl FromWorld for FurDrawNode {
    fn from_world(world: &mut World) -> Self {
        FurDrawNode {
            query: world.query(),
        }
    }
}

impl ViewNode for FurDrawNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewDepthTexture,
        &'static ViewUniformOffset,
    );

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (target, depth, view_offset): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let Some(pipelines) = world.get_resource::<FurPipelines>() else {
            return Ok(());
        };
        let Some(pipeline) = world
            .resource::<PipelineCache>()
            .get_render_pipeline(pipelines.draw_id)
        else {
            return Ok(());
        };

        let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("fur_draw"),
            color_attachments: &[Some(target.get_color_attachment())],
            depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_render_pipeline(pipeline);

        for buffers in self.query.iter_manual(world) {
            let Some(view_bg) = &buffers.draw_view_bind_group else {
                continue;
            };
            pass.set_bind_group(0, &buffers.draw_verts_bind_group, &[]);
            pass.set_bind_group(1, view_bg, &[view_offset.offset]);
            pass.draw(
                0..buffers.triangle_count * buffers.mode.verts_per_tri(),
                0..1,
            );
        }
        Ok(())
    }
}
