use crate::{gameplay::components::UiRoot, AssetHolder};
use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct TextButton;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct TextButtonRadial;

pub fn spawn_button<'w, 's, 'a, 'b, S: Into<String>>(
    child_builder: &'b mut ChildBuilder<'w, 's, 'a>,
    button_text: S,
    asset_holder: &AssetHolder,
) -> EntityCommands<'w, 's, 'b> {
    let mut e = child_builder.spawn_bundle(ButtonBundle {
        style: Style {
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            size: Size::new(Val::Auto, Val::Px(40.)),
            margin: UiRect {
                top: Val::Px(4.),
                bottom: Val::Px(4.),
                left: Val::Px(4.),
                right: Val::Px(4.),
            },
            ..default()
        },
        color: UiColor(Color::NONE),
        ..default()
    });

    e.insert(TextButton);

    e.with_children(|button| {
        button
            .spawn_bundle(ImageBundle {
                image: UiImage(asset_holder.button_radial.clone()),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        left: Val::Percent(15.),
                        top: Val::Percent(15.),
                        ..default()
                    },
                    size: Size::new(Val::Percent(70.), Val::Percent(70.)),
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                ..default()
            })
            .insert(TextButtonRadial);

        button
            .spawn_bundle(TextBundle::from_section(
                button_text,
                TextStyle {
                    font: asset_holder.font.clone(),
                    font_size: 16.,
                    color: Color::WHITE,
                },
            ))
            .insert(Style {
                margin: UiRect {
                    top: Val::Px(4.),
                    bottom: Val::Px(4.),
                    ..default()
                },
                ..default()
            });

        button.spawn_bundle(ImageBundle {
            image: UiImage(asset_holder.button_underline.clone()),
            style: Style {
                min_size: Size::new(Val::Percent(50.), Val::Px(16.)),
                ..default()
            },
            focus_policy: FocusPolicy::Pass,
            ..default()
        });
    });

    e
}

pub fn debug_spawn_button(
    mut commands: Commands,
    asset_holder: Res<AssetHolder>,
    ui_root: Query<Entity, With<UiRoot>>,
) {
    commands.entity(ui_root.single()).with_children(|mut root| {
        spawn_button(&mut root, "#1", &asset_holder);
        spawn_button(&mut root, "help", &asset_holder);
        spawn_button(&mut root, "ooh this one is really long!!", &asset_holder);
        spawn_button(&mut root, "help", &asset_holder);
    });
}

pub fn text_button_interaction(
    text_buttons: Query<(Entity, &Interaction), (Changed<Interaction>, With<TextButton>)>,
    mut button_radials: Query<(&mut UiColor, &Parent), With<TextButtonRadial>>,
) {
    for (button_entity, interaction) in text_buttons.iter() {
        let (mut radial_color, _) = button_radials
            .iter_mut()
            .find(|(_, parent)| parent.get() == button_entity)
            .expect("button should have radial child");

        match interaction {
            Interaction::None => {
                *radial_color = UiColor(Color::NONE);
            }
            Interaction::Hovered => {
                *radial_color = UiColor(Color::WHITE);
            }
            Interaction::Clicked => {
                *radial_color = UiColor(Color::GRAY);
            }
        }
    }
}
