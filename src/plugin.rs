use bevy::app::PostStartup;
use bevy::ecs::{intern::Interned, schedule::ScheduleLabel};
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;

use crate::components::{ToonRuntimeState, ToonShaderDiagnostics};
use crate::{ToonExtension, ToonMaterial, load_shader, scene_replace, systems};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ToonShaderSystems {
    ReplaceSceneMaterials,
    SyncRuntimeParameters,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct ToonShaderPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl ToonShaderPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for ToonShaderPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for ToonShaderPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        load_shader(app);

        app.add_plugins(MaterialPlugin::<ToonMaterial>::default())
            .init_resource::<ToonRuntimeState>()
            .init_resource::<ToonShaderDiagnostics>()
            .register_type::<ToonDiffuseMode>()
            .register_type::<ToonExtension>()
            .register_type::<ToonRim>()
            .register_type::<ToonShaderDiagnostics>()
            .register_type::<ToonSpecular>()
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(self.deactivate_schedule, systems::deactivate_runtime)
            .configure_sets(
                self.update_schedule,
                (
                    ToonShaderSystems::ReplaceSceneMaterials,
                    ToonShaderSystems::SyncRuntimeParameters,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                (
                    scene_replace::clear_completed_scene_roots,
                    scene_replace::replace_scene_materials,
                    scene_replace::replace_direct_materials,
                )
                    .chain()
                    .in_set(ToonShaderSystems::ReplaceSceneMaterials)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::sync_runtime_parameters
                    .in_set(ToonShaderSystems::SyncRuntimeParameters)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::publish_diagnostics.after(ToonShaderSystems::SyncRuntimeParameters),
            );
    }
}

use crate::material::{ToonDiffuseMode, ToonRim, ToonSpecular};
