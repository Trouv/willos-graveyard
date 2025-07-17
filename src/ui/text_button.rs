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
pub fn spawn<'w, S: Into<String>>(
    child_builder: &mut ChildSpawnerCommands<'w>,
    button_text: S,
    asset_holder: &AssetHolder,
    margin: Val,
    font_size: FontSize,
) -> EntityCommands<'w> {
    // Assigning the initial spawn to a variable is important for being able to return the
    // EntityCommands
    let mut e = child_builder.spawn((
        Button,
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            width: Val::Auto,
            height: Val::Px(50.),
            margin: UiRect {
                top: margin,
                bottom: margin,
                left: margin,
                right: margin,
            },
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));

    // PreviousComponent for tracking interaction changes, useful for detecting button presses
    e.insert(TextButton)
        .insert(PreviousComponent::<Interaction>::default());

    e.with_children(|button| {
        // spawn the background/highlight radial
        button
            .spawn((
                ImageNode::new(asset_holder.button_radial.clone()),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(15.),
                    top: Val::Percent(15.),
                    width: Val::Percent(70.),
                    height: Val::Percent(70.),
                    ..default()
                },
                FocusPolicy::Pass,
            ))
            .insert(ButtonRadial);

        // spawn the text
        button
            .spawn((
                Text::from_section(button_text),
                TextFont::new(asset_holder.font.clone()),
                TextColor::new(Color::WHITE),
            ))
            .insert(FontScale::from(font_size))
            .insert(Node {
                margin: UiRect {
                    top: Val::Px(4.),
                    bottom: Val::Px(4.),
                    ..default()
                },
                ..default()
            });

        // spawn the underline decoration
        button.spawn((
            ImageNode::new(asset_holder.button_underline.clone()),
            Node {
                height: Val::Percent(100. / 3.),
                aspect_ratio: Some(4.),
                ..default()
            },
            FocusPolicy::Pass,
        ));
    });

    e
}
