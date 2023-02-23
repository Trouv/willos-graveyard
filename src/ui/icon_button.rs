use bevy::prelude::*;

use crate::ui_atlas_image::UiAtlasImage;

struct IconButtonPlugin;

impl Plugin for IconButtonPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

#[derive(Default, Debug, Component)]
pub struct IconButton {
    pub icon: UiAtlasImage,
}

struct ButtonRadial;

fn spawn_icon_button_elements(
    mut commands: Commands,
    icon_buttons: Query<(Entity, &IconButton), Changed<IconButton>>,
) {
    for (entity, icon_button) in &icon_buttons {
        commands.entity(entity).add_children(|parent| {
            // Radial
            todo!()
        });
    }
}
