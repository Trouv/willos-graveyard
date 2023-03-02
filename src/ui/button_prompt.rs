//! Plugin for displaying button prompts on UI buttons that can be triggered by a physical button.
//!
//! This functionality is supported by [UiAction]s and `leafwing_input_manager`'s `Actionlike`s.
//! The button needs to have a `UiAction<T>` where `T` is an `Actionlike`.
//! Furthermore, `InputMap<T>` needs to also exist as a resource.
//! Finally, [`ButtonPromptPlugin<T>`](ButtonPromptPlugin) needs to be added to the app.
use std::marker::PhantomData;

use bevy::{ecs::query::ReadOnlyWorldQuery, prelude::*, reflect::Enum};
use bevy_asset_loader::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::{prelude::*, user_input::InputKind};

use crate::{
    ui::action::UiAction,
    ui_atlas_image::{AtlasImageBundle, UiAtlasImage},
    utils::resource_changed,
};

/// Plugin for displaying button prompts on UI buttons that can be triggered by a physical button.
///
/// See [module-level docs](self) for more info.
#[derive(Default)]
pub struct ButtonPromptPlugin<T>
where
    T: Actionlike + Send + Sync + Clone + 'static,
{
    phantom_data: PhantomData<T>,
}

impl<T> ButtonPromptPlugin<T>
where
    T: Actionlike + Send + Sync + Clone + 'static,
{
    /// Basic constructor for [ButtonPromptPlugin]
    pub fn new() -> ButtonPromptPlugin<T> {
        ButtonPromptPlugin {
            phantom_data: PhantomData,
        }
    }
}

impl<T> Plugin for ButtonPromptPlugin<T>
where
    T: Actionlike + Send + Sync + Clone + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_system(
            spawn_button_prompt::<T, Changed<UiAction<T>>>
                .run_if_resource_exists::<ButtonPromptAssets>(),
        )
        .add_system(
            spawn_button_prompt::<T, ()>
                .run_if_resource_exists::<ButtonPromptAssets>()
                .run_if(resource_changed::<InputMap<T>>),
        );
    }
}

/// Asset collection for assets relevant to button prompts.
#[derive(Debug, AssetCollection, Resource)]
pub struct ButtonPromptAssets {
    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 16, rows = 11))]
    #[asset(path = "textures/key-code-icons.png")]
    key_code_icons: Handle<TextureAtlas>,
}

#[derive(Copy, Clone, Debug, Default, Component)]
struct ButtonPrompt;

fn spawn_button_prompt<T, F>(
    mut commands: Commands,
    actions: Query<(Entity, &UiAction<T>), F>,
    existing_prompts: Query<(Entity, &Parent), With<ButtonPrompt>>,
    input_map: Res<InputMap<T>>,
    assets: Res<ButtonPromptAssets>,
) where
    T: Actionlike + Send + Sync + Clone + 'static,
    F: ReadOnlyWorldQuery,
{
    for (entity, action) in &actions {
        // despawn any existing prompts
        existing_prompts
            .iter()
            .filter(|(_, p)| p.get() == entity)
            .for_each(|(e, _)| commands.entity(e).despawn_recursive());

        // spawn button prompt
        if let Some(UserInput::Single(InputKind::Keyboard(key_code))) = input_map
            .get((**action).clone())
            .iter()
            .find(|i| matches!(i, UserInput::Single(InputKind::Keyboard(_))))
        {
            commands.entity(entity).with_children(|parent| {
                parent
                    .spawn(AtlasImageBundle {
                        atlas_image: UiAtlasImage {
                            texture_atlas: assets.key_code_icons.clone(),
                            index: key_code.variant_index(),
                        },
                        image_bundle: ImageBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: UiRect {
                                    top: Val::Px(0.),
                                    left: Val::Px(0.),
                                    ..default()
                                },
                                size: Size::new(Val::Undefined, Val::Percent(25.)),
                                aspect_ratio: Some(1.),
                                ..default()
                            },
                            z_index: ZIndex::Local(10),
                            ..default()
                        },
                    })
                    .insert(ButtonPrompt);
            });
        }
    }
}
