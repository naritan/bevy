use super::{prepare::DlssRenderContext, Dlss, DlssFeature};
use bevy_camera::{Camera, MainPassResolutionOverride, Projection};
use bevy_ecs::{
    query::{Has, With},
    system::{Commands, Query, ResMut},
};
use tracing::info;
use bevy_render::{sync_world::RenderEntity, view::Hdr, MainWorld};

pub fn extract_dlss<F: DlssFeature>(
    mut commands: Commands,
    mut main_world: ResMut<MainWorld>,
    cleanup_query: Query<Has<Dlss<F>>>,
    context_query: Query<Has<DlssRenderContext<F>>>,
) {
    let mut cameras_3d = main_world
        .query_filtered::<(RenderEntity, &Camera, &Projection, Option<&mut Dlss<F>>), With<Hdr>>();

    for (entity, camera, camera_projection, mut dlss) in cameras_3d.iter_mut(&mut main_world) {
        let mut entity_commands = commands
            .get_entity(entity)
            .expect("Camera entity wasn't synced.");
        if let Some(ref mut dlss) = dlss {
            let has_context = context_query.get(entity).unwrap_or(false);

            if camera.is_active && camera_projection.is_perspective() {
                if dlss.reset {
                    info!("[DLSS extract] active camera reset=true has_context={}", has_context);
                }
                entity_commands.insert((**dlss).clone());
                if has_context || !dlss.reset {
                    dlss.reset = false;
                } else {
                    info!("[DLSS extract] keeping reset=true in main world (no context)");
                }
            } else if dlss.reset {
                info!("[DLSS extract] inactive camera reset=true has_context={}", has_context);
                if cleanup_query.get(entity) == Ok(true) {
                    entity_commands.insert((**dlss).clone());
                }
                if has_context {
                    dlss.reset = false;
                }
            }
        } else if cleanup_query.get(entity) == Ok(true) {
            entity_commands.remove::<(Dlss<F>, DlssRenderContext<F>, MainPassResolutionOverride)>();
        }
    }
}
