use crate::{gameplay::components::UiRoot, AssetHolder};
use bevy::{ecs::system::EntityCommands, prelude::*};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct ButtonText;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct ButtonRadial;

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
            min_size: Size::new(Val::Px(64.), Val::Px(32.)),

            ..default()
        },
        image: UiImage(asset_holder.button_radial.clone()),
        ..default()
    });

    e.insert(ButtonRadial);

    e.with_children(|button| {
        button
            .spawn_bundle(TextBundle::from_section(
                button_text,
                TextStyle {
                    font: asset_holder.font.clone(),
                    font_size: 16.,
                    color: Color::WHITE,
                },
            ))
            .insert(ButtonText);

        button.spawn_bundle(ImageBundle {
            image: UiImage(asset_holder.button_underline.clone()),
            style: Style {
                min_size: Size::new(Val::Px(64.), Val::Px(32.)),
                ..default()
            },
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
        spawn_button(&mut root, "help", &asset_holder);
        spawn_button(&mut root, "help", &asset_holder);
        spawn_button(&mut root, "ooh this one is really long!!", &asset_holder);
        spawn_button(&mut root, "help", &asset_holder);
    });
}
