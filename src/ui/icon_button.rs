//! Plugin for building "icon buttons" in the style of this game.
use bevy::{prelude::*, ui::FocusPolicy};
use bevy_asset_loader::prelude::AssetCollection;

use crate::{
    previous_component::PreviousComponent, ui::button_radial::ButtonRadial,
    ui_atlas_image::UiAtlasImage, GameState,
};

/// System label for systems that respond to `IconButton` changes.
#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
pub struct IconButtonSet;

/// Plugin for building "icon buttons" in the style of this game.
///
/// Use [IconButtonBundle::new] to get started.
pub struct IconButtonPlugin;

impl Plugin for IconButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_icon_button_elements
                .run_if(not(in_state(GameState::AssetLoading)))
                .in_set(IconButtonSet),
        );
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
    /// Button with all other elements, but no icon.
    NoIcon,
    /// Use a simple Image for the button's icon.
    ImageIcon(UiImage),
    /// Use a texture atlas + index for the button's icon.
    AtlasImageIcon(UiAtlasImage),
}

// PartialEq is useful for testing, but UiImage does not implement it.
// So, this trivial manual implementation is provided.
impl PartialEq for IconButton {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (IconButton::NoIcon, IconButton::NoIcon) => true,
            (
                IconButton::ImageIcon(UiImage { texture: s, .. }),
                IconButton::ImageIcon(UiImage { texture: o, .. }),
            ) => s == o,
            (IconButton::AtlasImageIcon(s), IconButton::AtlasImageIcon(o)) => s == o,
            _ => false,
        }
    }
}

impl Eq for IconButton {}

#[derive(Debug, Component)]
struct IconButtonElement;

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
    /// Constructor for the plugin with no additional styling.
    pub fn new(icon_button: IconButton) -> IconButtonBundle {
        IconButtonBundle {
            icon_button,
            button_bundle: ButtonBundle {
                style: Style {
                    flex_grow: 1.,
                    aspect_ratio: Some(1.),
                    ..default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
            previous_interaction: PreviousComponent::<Interaction>::default(),
        }
    }

    /// Constructor for the bundle that applies the given size to the styling.
    pub fn new_with_size(icon_button: IconButton, width: Val, height: Val) -> IconButtonBundle {
        IconButtonBundle {
            icon_button,
            button_bundle: ButtonBundle {
                style: Style {
                    width,
                    height,
                    aspect_ratio: Some(1.),
                    ..default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
            previous_interaction: PreviousComponent::<Interaction>::default(),
        }
    }

    /// Constructor for the bundle that applies the given position to the styling.
    pub fn new_with_absolute_position(
        icon_button: IconButton,
        position: UiRect,
    ) -> IconButtonBundle {
        let UiRect {
            left,
            right,
            top,
            bottom,
        } = position;
        IconButtonBundle {
            icon_button,
            button_bundle: ButtonBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left,
                    right,
                    top,
                    bottom,
                    aspect_ratio: Some(1.),
                    ..default()
                },
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
    icon_button_elements: Query<(Entity, &Parent), With<IconButtonElement>>,
    assets: Res<IconButtonAssets>,
) {
    for (entity, icon_button) in &icon_buttons {
        icon_button_elements
            .iter()
            .filter(|(_, p)| p.get() == entity)
            .for_each(|(e, _)| commands.entity(e).despawn_recursive());

        commands.entity(entity).with_children(|parent| {
            // Radial
            parent
                .spawn(ButtonRadial)
                .insert(ImageBundle {
                    image: UiImage::new(assets.radial.clone()),
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(12.5),
                        right: Val::Percent(12.5),
                        top: Val::Percent(12.5),
                        bottom: Val::Percent(12.5),
                        ..default()
                    },
                    focus_policy: FocusPolicy::Pass,
                    background_color: BackgroundColor(Color::NONE),
                    ..default()
                })
                .insert(IconButtonElement);

            // Outline
            parent
                .spawn(ImageBundle {
                    image: UiImage::new(assets.outline.clone()),
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(0.),
                        right: Val::Percent(0.),
                        top: Val::Percent(0.),
                        bottom: Val::Percent(0.),
                        ..default()
                    },
                    focus_policy: FocusPolicy::Pass,
                    ..default()
                })
                .insert(IconButtonElement);

            // Icon
            let mut icon_entity = parent.spawn(ImageBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.),
                    right: Val::Percent(0.),
                    top: Val::Percent(0.),
                    bottom: Val::Percent(0.),
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                ..default()
            });

            icon_entity.insert(IconButtonElement);

            match icon_button {
                IconButton::AtlasImageIcon(i) => icon_entity.insert(i.clone()),
                IconButton::ImageIcon(i) => icon_entity.insert(i.clone()),
                _ => icon_entity.insert(BackgroundColor(Color::NONE)),
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
            .add_state::<GameState>()
            .insert_resource(NextState(Some(GameState::LevelTransition)));

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

    fn spawn_icon_button(app: &mut App, icon_button: IconButton) -> Entity {
        app.world.spawn(IconButtonBundle::new(icon_button)).id()
    }

    #[test]
    fn elements_spawned_with_ui_image() {
        let mut app = app_setup();
        let asset_collection = asset_collection_setup(&mut app);

        let icon = Handle::weak(HandleId::random::<Image>());
        let icon_button_entity =
            spawn_icon_button(&mut app, IconButton::ImageIcon(UiImage::new(icon.clone())));

        app.update();

        let mut children = app.world.query::<(&UiImage, &Parent)>();

        let children: Vec<_> = children
            .iter(&app.world)
            .filter(|(_, p)| p.get() == icon_button_entity)
            .map(|(i, _)| i)
            .collect();

        assert_eq!(children.len(), 3);

        assert_eq!(children[0].texture, asset_collection.radial);

        assert_eq!(children[1].texture, asset_collection.outline);

        assert_eq!(children[2].texture, icon);
    }

    #[test]
    fn elements_spawned_with_ui_atlas_image() {
        let mut app = app_setup();
        let asset_collection = asset_collection_setup(&mut app);

        let icon = Handle::weak(HandleId::random::<TextureAtlas>());
        let icon_button_entity = spawn_icon_button(
            &mut app,
            IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: icon.clone(),
                index: 2,
            }),
        );

        app.update();

        let mut children = app
            .world
            .query::<(Option<&UiAtlasImage>, &UiImage, &Parent)>();

        let children: Vec<_> = children
            .iter(&app.world)
            .filter(|(.., p)| p.get() == icon_button_entity)
            .map(|(a, i, _)| (a, i))
            .collect();

        assert_eq!(children.len(), 3);

        assert_eq!(children[0].1.texture, asset_collection.radial);

        assert_eq!(children[1].1.texture, asset_collection.outline);

        assert_eq!(children[2].0.unwrap().texture_atlas, icon);
    }

    #[test]
    fn elements_can_change() {
        let mut app = app_setup();
        asset_collection_setup(&mut app);

        let first_icon = Handle::weak(HandleId::random::<Image>());
        let icon_button_entity = spawn_icon_button(
            &mut app,
            IconButton::ImageIcon(UiImage::new(first_icon.clone())),
        );

        app.update();

        let mut children = app.world.query::<(&UiImage, &Parent)>();

        let children: Vec<_> = children
            .iter(&app.world)
            .filter(|(_, p)| p.get() == icon_button_entity)
            .map(|(i, _)| i)
            .collect();

        assert_eq!(children.len(), 3);

        assert_eq!(children[2].texture, first_icon);

        // Change the component
        let second_icon = Handle::weak(HandleId::random::<Image>());
        *app.world
            .entity_mut(icon_button_entity)
            .get_mut::<IconButton>()
            .unwrap() = IconButton::ImageIcon(UiImage::new(second_icon.clone()));

        app.update();

        let mut children = app.world.query::<(&UiImage, &Parent)>();

        let children: Vec<_> = children
            .iter(&app.world)
            .filter(|(_, p)| p.get() == icon_button_entity)
            .map(|(i, _)| i)
            .collect();

        assert_eq!(children.len(), 3);

        assert_eq!(children[2].texture, second_icon);
    }

    #[test]
    fn update_does_not_despawn_greedily() {
        let mut app = app_setup();
        asset_collection_setup(&mut app);

        let first_icon = Handle::weak(HandleId::random::<Image>());

        let icon_button_entity = spawn_icon_button(
            &mut app,
            IconButton::ImageIcon(UiImage::new(first_icon.clone())),
        );

        let additional_child_entity = app.world.spawn_empty().id();

        app.world
            .entity_mut(icon_button_entity)
            .push_children(&[additional_child_entity]);

        app.update();

        let mut children = app.world.query::<(Entity, &Parent)>();

        let children: Vec<_> = children
            .iter(&app.world)
            .filter(|(_, p)| p.get() == icon_button_entity)
            .map(|(e, _)| e)
            .collect();

        assert_eq!(children.len(), 4);
        assert!(children.contains(&additional_child_entity));
    }
}
