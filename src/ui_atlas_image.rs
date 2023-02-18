use bevy::prelude::*;

pub struct UiAtlasImagePlugin;

impl Plugin for UiAtlasImagePlugin {
    fn build(&self, app: &mut App) {
        todo!();
    }
}

#[derive(Deref, DerefMut, Resource)]
struct UiAtlasImageMap(HashMap<Handle<TextureAtlas>, Vec<Handle<Image>>>);

struct UiAtlasImage {
    texture_atlas: Handle<TextureAtlas>,
    index: usize,
}
