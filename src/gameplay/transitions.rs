use crate::gameplay::components::*;
use bevy::prelude::*;

pub fn spawn_ui_root(mut commands: Commands) {
    commands
        .spawn_bundle(NodeBundle {
            color: UiColor(Color::NONE),
            style: Style {
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(UiRoot);
}
