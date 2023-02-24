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
        commands.entity(entity).add_children(|parent| {
            // Radial
            todo!()
        });
    }
}
