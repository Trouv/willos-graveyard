use bevy::{prelude::*, ui::FocusPolicy};
use bevy_asset_loader::prelude::AssetCollection;
use iyes_loopless::prelude::*;

use crate::{
    previous_component::PreviousComponent,
    ui::button_radial::ButtonRadial,
    ui_atlas_image::{AtlasImageBundle, UiAtlasImage, UiAtlasImagePlugin},
    GameState,
};

pub struct IconButtonPlugin;

impl Plugin for IconButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_icon_button_elements.run_not_in_state(GameState::AssetLoading))
            .add_plugin(UiAtlasImagePlugin)
            .add_enter_system(
                crate::GameState::LevelTransition,
                |mut commands: Commands, assets: Res<IconButtonAssets>| {
                    commands.spawn(IconButtonBundle::new(
                        IconButton::AtlasImageIcon(UiAtlasImage {
                            texture_atlas: assets.ui_moves.clone(),
                            index: 5,
                        }),
                        Val::Px(128.),
                    ));
                },
            );
    }
}

#[derive(Debug, Component)]
pub enum IconButton {
    ImageIcon(UiImage),
    AtlasImageIcon(UiAtlasImage),
}

#[derive(Debug, Bundle)]
pub struct IconButtonBundle {
    icon_button: IconButton,
    button_bundle: ButtonBundle,
    previous_interaction: PreviousComponent<Interaction>,
}

impl IconButtonBundle {
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

#[derive(Default, Debug, AssetCollection, Resource)]
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
