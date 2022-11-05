//! Contains functions, systems, and components related to "text button"s

use crate::{
    previous_component::PreviousComponent,
    ui::font_scale::{FontScale, FontSize},
    AssetHolder,
};
use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};

/// Marker component for the main "text button" ui node.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct TextButton;

/// Marker component for the background highlight radial on "text button"s
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct TextButtonRadial;

/// Spawns a text button with the provided `button_text`.
///
/// Returns [EntityCommands] for the button entity.
/// You can use this to add more components if necessary.
///
/// To give this button simple functionality, consider inserting a [crate::ui::actions::UiAction].
#[allow(dead_code)]
pub fn spawn<'w, 's, 'a, 'b, S: Into<String>>(
    child_builder: &'b mut ChildBuilder<'w, 's, 'a>,
    button_text: S,
    asset_holder: &AssetHolder,
    margin: Val,
    font_size: FontSize,
) -> EntityCommands<'w, 's, 'b> {
    // Assigning the initial spawn to a variable is important for being able to return the
    // EntityCommands
    let mut e = child_builder.spawn_bundle(ButtonBundle {
        style: Style {
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            size: Size::new(Val::Auto, Val::Px(40.)),
            margin: UiRect {
                top: margin,
                bottom: margin,
                left: margin,
                right: margin,
            },
            ..default()
        },
        color: UiColor(Color::NONE),
        ..default()
    });

    // PreviousComponent for tracking interaction changes, useful for detecting button presses
    e.insert(TextButton)
        .insert(PreviousComponent::<Interaction>::default());

    e.with_children(|button| {
        // spawn the background/highlight radial
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

        // spawn the text
        button
            .spawn_bundle(TextBundle::from_section(
                button_text,
                TextStyle {
                    font: asset_holder.font.clone(),
                    font_size: 16.,
                    color: Color::WHITE,
                },
            ))
            .insert(FontScale::from(font_size))
            .insert(Style {
                margin: UiRect {
                    top: Val::Px(4.),
                    bottom: Val::Px(4.),
                    ..default()
                },
                ..default()
            });

        // spawn the underline decoration
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

/// System that alters the visuals of a text button to show interaction
pub(super) fn text_button_visuals(
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

#[cfg(feature = "ui-debug")]
pub mod debug {
    use super::*;
    use crate::{gameplay::components::UiRoot, ui::actions::UiAction};

    pub fn debug_spawn_button(
        mut commands: Commands,
        asset_holder: Res<AssetHolder>,
        ui_root: Query<Entity, With<UiRoot>>,
    ) {
        commands.entity(ui_root.single()).with_children(|mut root| {
            spawn(&mut root, "#1", &asset_holder, Val::Px(4.), FontSize::Small)
                .insert(UiAction::Debug("#1"));
            spawn(
                &mut root,
                "help",
                &asset_holder,
                Val::Px(4.),
                FontSize::Small,
            )
            .insert(UiAction::Debug("Help 1"));
            spawn(
                &mut root,
                "ooh this one is really long!!",
                &asset_holder,
                Val::Px(4.),
                FontSize::Small,
            )
            .insert(UiAction::Debug("long"));
            spawn(
                &mut root,
                "help",
                &asset_holder,
                Val::Px(4.),
                FontSize::Small,
            )
            .insert(UiAction::Debug("Help 2"));
        });
    }
}
