//! Plugin for displaying button prompts on UI buttons that can be triggered by a physical button.
//!
//! This functionality is supported by [UiAction]s and `leafwing_input_manager`'s `Actionlike`s.
//! The button needs to have a `UiAction<T>` where `T` is an `Actionlike`.
//! Furthermore, `InputMap<T>` needs to also exist as a resource.
//! Finally, [`ButtonPromptPlugin<T>`](ButtonPromptPlugin) needs to be added to the app.
use std::marker::PhantomData;

use bevy::{ecs::query::QueryFilter, prelude::*, reflect::Enum};
use bevy_asset_loader::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{ui::action::UiAction, ui_atlas_image::UiAtlasImage, utils::resource_changed};

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
        app.add_systems(
            Update,
            (
                spawn_button_prompt::<T, Changed<UiAction<T>>>
                    .run_if(resource_exists::<ButtonPromptAssets>),
                spawn_button_prompt::<T, ()>
                    .run_if(resource_exists::<ButtonPromptAssets>)
                    .run_if(resource_changed::<InputMap<T>>),
            ),
        );
    }
}

/// Asset collection for assets relevant to button prompts.
#[derive(Clone, Debug, AssetCollection, Resource)]
pub struct ButtonPromptAssets {
    #[asset(texture_atlas(tile_size_x = 16, tile_size_y = 16, columns = 16, rows = 11))]
    key_code_icons_atlas: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/key-code-icons.png")]
    key_code_icons_image: Handle<Image>,
}

#[derive(Copy, Clone, Debug, Default, Component)]
struct ButtonPrompt;

fn spawn_button_prompt<T, F>(
    mut commands: Commands,
    actions: Query<(Entity, &UiAction<T>), F>,
    existing_prompts: Query<(Entity, &ChildOf), With<ButtonPrompt>>,
    input_map: Res<InputMap<T>>,
    assets: Res<ButtonPromptAssets>,
) where
    T: Actionlike + Send + Sync + Clone + 'static,
    F: QueryFilter,
{
    for (entity, action) in &actions {
        // despawn any existing prompts
        existing_prompts
            .iter()
            .filter(|(_, p)| p.parent() == entity)
            .for_each(|(e, _)| commands.entity(e).despawn());

        // spawn button prompt
        if let Some(key_code) = input_map
            .get_buttonlike(action)
            .iter()
            .flat_map(|inputs| inputs.iter())
            .find_map(|i| i.as_any().downcast_ref::<KeyCode>())
        {
            commands.entity(entity).with_children(|parent| {
                parent
                    .spawn((
                        UiAtlasImage {
                            image: assets.key_code_icons_image.clone(),
                            texture_atlas: assets.key_code_icons_atlas.clone(),
                            index: key_code.variant_index(),
                        },
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(0.),
                            left: Val::Px(0.),
                            height: Val::Percent(25.),
                            aspect_ratio: Some(1.),
                            ..default()
                        },
                        ZIndex(10),
                    ))
                    .insert(ButtonPrompt);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::action::UiActionPlugin;
    use rand::prelude::*;

    use super::*;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Reflect, Actionlike)]
    enum MyAction {
        Jump,
        Shoot,
    }

    fn app_setup() -> App {
        let mut app = App::new();

        app.add_plugins((
            UiActionPlugin::<MyAction>::new(),
            ButtonPromptPlugin::<MyAction>::new(),
        ));

        app
    }

    fn asset_setup(app: &mut App) -> ButtonPromptAssets {
        let mut rng = rand::thread_rng();
        let button_prompt_assets = ButtonPromptAssets {
            key_code_icons_atlas: Handle::weak_from_u128(rng.gen()),
        };

        app.insert_resource(button_prompt_assets.clone());

        button_prompt_assets
    }

    fn input_map_setup(app: &mut App) {
        let input_map = InputMap::<MyAction>::new([
            (KeyCode::Space, MyAction::Jump),
            (KeyCode::F, MyAction::Shoot),
        ]);

        app.insert_resource(input_map);
    }

    fn spawn_button(app: &mut App, action: MyAction) -> Entity {
        app.world
            .spawn(UiAction(action))
            .insert(ButtonBundle::default())
            .id()
    }

    fn get_child_component<C: Component>(app: &mut App, entity: Entity) -> &C {
        let mut children = app.world.query::<(&C, &Parent)>();

        children
            .iter(&app.world)
            .find(|(_, p)| p.get() == entity)
            .map(|(c, _)| c)
            .unwrap()
    }

    #[test]
    fn button_prompt_displays_correct_key() {
        let mut app = app_setup();
        let assets = asset_setup(&mut app);
        input_map_setup(&mut app);

        let jump_entity = spawn_button(&mut app, MyAction::Jump);
        let shoot_entity = spawn_button(&mut app, MyAction::Shoot);

        app.update();

        let jump_button_prompt = get_child_component::<UiAtlasImage>(&mut app, jump_entity);

        assert_eq!(
            *jump_button_prompt,
            UiAtlasImage {
                texture_atlas: assets.key_code_icons_atlas.clone(),
                index: 76
            }
        );

        let shoot_button_prompt = get_child_component::<UiAtlasImage>(&mut app, shoot_entity);

        assert_eq!(
            *shoot_button_prompt,
            UiAtlasImage {
                texture_atlas: assets.key_code_icons_atlas,
                index: 15
            }
        );
    }

    #[test]
    fn button_prompt_changes_with_action() {
        let mut app = app_setup();
        let assets = asset_setup(&mut app);
        input_map_setup(&mut app);

        // check its original value image selection
        let button_entity = spawn_button(&mut app, MyAction::Jump);

        app.update();

        let button_prompt = get_child_component::<UiAtlasImage>(&mut app, button_entity);

        assert_eq!(
            *button_prompt,
            UiAtlasImage {
                texture_atlas: assets.key_code_icons_atlas.clone(),
                index: 76
            }
        );

        // change its action and check its image again

        *app.world
            .entity_mut(button_entity)
            .get_mut::<UiAction<MyAction>>()
            .unwrap() = UiAction(MyAction::Shoot);

        app.update();

        let button_prompt = get_child_component::<UiAtlasImage>(&mut app, button_entity);

        assert_eq!(
            *button_prompt,
            UiAtlasImage {
                texture_atlas: assets.key_code_icons_atlas.clone(),
                index: 15
            }
        );
    }

    #[test]
    fn button_prompt_changes_with_input_map() {
        let mut app = app_setup();
        let assets = asset_setup(&mut app);
        input_map_setup(&mut app);

        // check its original value image selection
        let button_entity = spawn_button(&mut app, MyAction::Jump);

        app.update();

        let button_prompt = get_child_component::<UiAtlasImage>(&mut app, button_entity);

        assert_eq!(
            *button_prompt,
            UiAtlasImage {
                texture_atlas: assets.key_code_icons_atlas.clone(),
                index: 76
            }
        );

        // change the input map and check its image again

        let mut input_map = app.world.get_resource_mut::<InputMap<MyAction>>().unwrap();
        input_map.clear_action(MyAction::Jump);
        input_map.insert(
            UserInput::Single(InputKind::Keyboard(KeyCode::W)),
            MyAction::Jump,
        );

        app.update();

        let button_prompt = get_child_component::<UiAtlasImage>(&mut app, button_entity);

        assert_eq!(
            *button_prompt,
            UiAtlasImage {
                texture_atlas: assets.key_code_icons_atlas.clone(),
                index: 32
            }
        );
    }
}
