use crate::{
    nine_slice::{
        generate_nineslice_image, texture_atlas_from_nine_slice, NineSliceIndex, NineSliceSize,
    },
    ui::{actions::UiAction, text_button},
    AssetHolder,
};
use bevy::prelude::*;
use bevy_easings::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;
use std::time::Duration;

pub struct LevelSelectPlugin;

impl Plugin for LevelSelectPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Component)]
pub struct LevelSelect;

fn spawn_level_select_card(
    mut commands: Commands,
    asset_holder: Res<AssetHolder>,
    mut images: ResMut<Assets<Image>>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
) {
    // TODO: refactor this to avoid repeated code with spawn_level_card
    let level_card_atlas = texture_atlas_from_nine_slice(
        asset_holder.tarot_sheet.clone(),
        Vec2::splat(64.),
        16.,
        16.,
        16.,
        16.,
    );
    let level_card_texture = generate_nineslice_image(
        NineSliceSize {
            inner_width: 8,
            inner_height: 4,
        },
        NineSliceIndex::default(),
        &level_card_atlas,
        &mut images,
    )
    .unwrap();
    commands
        .spawn_bundle(ImageBundle {
            image: UiImage(level_card_texture),
            //visibility: Visibility { is_visible: false },
            ..default()
        })
        .insert(
            Style {
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                position_type: PositionType::Absolute,
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                position: UiRect {
                    top: Val::Percent(100.),
                    left: Val::Percent(0.),
                    ..default()
                },
                ..default()
            }
            .ease_to(
                Style {
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::ColumnReverse,
                    position_type: PositionType::Absolute,
                    size: Size {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                    },
                    position: UiRect {
                        top: Val::Percent(0.),
                        left: Val::Percent(0.),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                EaseFunction::QuadraticOut,
                EasingType::Once {
                    duration: Duration::from_secs(1),
                },
            ),
        )
        .insert(LevelSelect)
        .with_children(|parent| {
            // spawn title
            parent.spawn_bundle(TextBundle {
                text: Text::from_section(
                    "Level Select",
                    TextStyle {
                        font: asset_holder.font.clone(),
                        font_size: 50.,
                        color: Color::WHITE,
                    },
                )
                .with_alignment(TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                }),
                style: Style {
                    margin: UiRect {
                        top: Val::Px(10.),
                        bottom: Val::Px(10.),
                        left: Val::Px(10.),
                        right: Val::Px(10.),
                    },
                    ..default()
                },
                ..default()
            });

            // spawn level button container
            parent
                .spawn_bundle(NodeBundle {
                    color: UiColor(Color::NONE),
                    style: Style {
                        flex_wrap: FlexWrap::Wrap,
                        justify_content: JustifyContent::Center,
                        margin: UiRect {
                            top: Val::Px(10.),
                            bottom: Val::Px(10.),
                            left: Val::Px(10.),
                            right: Val::Px(10.),
                        },
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    if let Some(ldtk) = ldtk_assets.get(&asset_holder.ldtk) {
                        for (i, _) in ldtk.iter_levels().enumerate() {
                            text_button::spawn(parent, format!("#{}", i + 1), &asset_holder)
                                .insert(UiAction::GoToLevel(LevelSelection::Index(i)));
                        }
                    }
                });
        });
}
