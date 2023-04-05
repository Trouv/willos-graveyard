//! Contains functions, systems, and components related to "text button"s

use crate::{
    previous_component::PreviousComponent,
    ui::{
        button_radial::ButtonRadial,
        font_scale::{FontScale, FontSize},
    },
    AssetHolder,
};
use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};

/// Marker component for the main "text button" ui node.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct TextButton;

/// Spawns a text button with the provided `button_text`.
///
/// Returns [EntityCommands] for the button entity.
/// You can use this to add more components if necessary.
///
/// To give this button simple functionality, consider inserting a [crate::ui::actions::UiAction].
#[allow(dead_code)]
pub fn spawn<'w, 's, 'a, S: Into<String>>(
    child_builder: &'a mut ChildBuilder<'w, 's, '_>,
    button_text: S,
    asset_holder: &AssetHolder,
    margin: Val,
    font_size: FontSize,
) -> EntityCommands<'w, 's, 'a> {
    // Assigning the initial spawn to a variable is important for being able to return the
    // EntityCommands
    let mut e = child_builder.spawn(ButtonBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            size: Size::new(Val::Auto, Val::Px(50.)),
            margin: UiRect {
                top: margin,
                bottom: margin,
                left: margin,
                right: margin,
            },
            ..default()
        },
        background_color: BackgroundColor(Color::NONE),
        ..default()
    });

    // PreviousComponent for tracking interaction changes, useful for detecting button presses
    e.insert(TextButton)
        .insert(PreviousComponent::<Interaction>::default());

    e.with_children(|button| {
        // spawn the background/highlight radial
        button
            .spawn(ImageBundle {
                image: UiImage::new(asset_holder.button_radial.clone()),
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
            .insert(ButtonRadial);

        // spawn the text
        button
            .spawn(TextBundle::from_section(
                button_text,
                TextStyle {
                    font: asset_holder.font.clone(),
                    color: Color::WHITE,
                    ..default()
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
        button.spawn(ImageBundle {
            image: UiImage::new(asset_holder.button_underline.clone()),
            style: Style {
                size: Size::height(Val::Percent(100. / 3.)),
                aspect_ratio: Some(4.),
                ..default()
            },
            focus_policy: FocusPolicy::Pass,
            ..default()
        });
    });

    e
}
