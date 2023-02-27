//! Plugin for building "icon buttons" in the style of this game.
use bevy::{prelude::*, ui::FocusPolicy};
use bevy_asset_loader::prelude::AssetCollection;
use iyes_loopless::prelude::*;

use crate::{
    previous_component::PreviousComponent, ui::button_radial::ButtonRadial,
    ui_atlas_image::UiAtlasImage, GameState,
};

/// Plugin for building "icon buttons" in the style of this game.
///
/// Use [IconButtonBundle::new] to get started.
pub struct IconButtonPlugin;

impl Plugin for IconButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_icon_button_elements.run_not_in_state(GameState::AssetLoading));
    }
}

/// Component acting as a "blueprint" for building icon buttons.
///
/// Stores all the metadata needed for building an icon button.
/// The plugin will flesh out the entity/its children in a system later.
///
/// Currently, the only metadata needed is the image to use as an icon.
#[derive(Debug, Component)]
pub enum IconButton {
    /// Use a simple Image for the button's icon.
    ImageIcon(UiImage),
    /// Use a texture atlas + index for the button's icon.
    AtlasImageIcon(UiAtlasImage),
}

/// Bundle containing all components necessary for a functional IconButton.
///
/// You will need to insert a [crate::ui::actions::UiAction] separately.
#[derive(Debug, Bundle)]
pub struct IconButtonBundle {
    icon_button: IconButton,
    button_bundle: ButtonBundle,
    previous_interaction: PreviousComponent<Interaction>,
}

impl IconButtonBundle {
    /// Constructor for the bundle that applies the appropriate styling for you.
    pub fn new(icon_button: IconButton, diameter: Val) -> IconButtonBundle {
        IconButtonBundle {
            icon_button,
            button_bundle: ButtonBundle {
                style: Style {
                    size: Size {
                        width: diameter,
                        height: diameter,
                    },
                    ..default()
                },
                //interaction: Interaction::None,
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
            previous_interaction: PreviousComponent::<Interaction>::default(),
        }
    }
}

/// Asset collection for loading assets relevant to icon buttons.
#[derive(Clone, Default, Debug, AssetCollection, Resource)]
pub struct IconButtonAssets {
    #[asset(path = "textures/icon-button-outline.png")]
    outline: Handle<Image>,
    #[asset(path = "textures/icon-button-radial.png")]
    radial: Handle<Image>,
}

fn spawn_icon_button_elements(
    mut commands: Commands,
    icon_buttons: Query<(Entity, &IconButton), Changed<IconButton>>,
    assets: Res<IconButtonAssets>,
) {
    for (entity, icon_button) in &icon_buttons {
        commands.entity(entity).despawn_descendants();

        commands.entity(entity).add_children(|parent| {
            // Radial
            parent.spawn(ButtonRadial).insert(ImageBundle {
                image: UiImage(assets.radial.clone()),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect::all(Val::Percent(12.5)),
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                background_color: BackgroundColor(Color::NONE),
                ..default()
            });

            // Outline
            parent.spawn(ImageBundle {
                image: UiImage(assets.outline.clone()),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect::all(Val::Percent(0.)),
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                ..default()
            });

            // Icon
            let mut icon_entity = parent.spawn(ImageBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect::all(Val::Percent(0.)),
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                ..default()
            });

            match icon_button {
                IconButton::AtlasImageIcon(i) => icon_entity.insert(i.clone()),
                IconButton::ImageIcon(i) => icon_entity.insert(i.clone()),
            };
        });
    }
}

#[cfg(test)]
mod tests {
    use bevy::asset::HandleId;

    use super::*;

    fn app_setup() -> App {
        let mut app = App::new();

        app.add_plugin(IconButtonPlugin)
            .add_plugin(HierarchyPlugin)
            .add_loopless_state(GameState::LevelTransition);

        app
    }

    fn asset_collection_setup(app: &mut App) -> IconButtonAssets {
        let assets = IconButtonAssets {
            outline: Handle::weak(HandleId::random::<Image>()),
            radial: Handle::weak(HandleId::random::<Image>()),
        };

        app.insert_resource(assets.clone());

        assets
    }

    #[test]
    fn elements_spawned_with_ui_image() {
        let mut app = app_setup();
        let asset_collection = asset_collection_setup(&mut app);

        let icon = Handle::weak(HandleId::random::<Image>());
        let icon_button_entity = app
            .world
            .spawn(IconButtonBundle::new(
                IconButton::ImageIcon(UiImage(icon.clone())),
                Val::Px(50.),
            ))
            .id();

        app.update();

        let mut children = app.world.query::<(Entity, &Parent)>();

        let children: Vec<_> = children
            .iter(&app.world)
            .filter(|(_, p)| p.get() == icon_button_entity)
            .collect();

        assert_eq!(children.len(), 3);

        assert_eq!(
            app.world.entity(children[0].0).get::<UiImage>().unwrap().0,
            asset_collection.radial
        );

        assert_eq!(
            app.world.entity(children[1].0).get::<UiImage>().unwrap().0,
            asset_collection.outline
        );

        assert_eq!(
            app.world.entity(children[2].0).get::<UiImage>().unwrap().0,
            icon
        );
    }
}
