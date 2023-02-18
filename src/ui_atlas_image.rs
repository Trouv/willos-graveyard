use bevy::prelude::*;
use std::collections::HashMap;

pub struct UiAtlasImagePlugin;

impl Plugin for UiAtlasImagePlugin {
    fn build(&self, app: &mut App) {
        todo!();
    }
}

#[derive(Deref, DerefMut, Resource)]
struct UiAtlasImageMap(HashMap<Handle<TextureAtlas>, Vec<Handle<Image>>>);

#[derive(Component)]
pub struct UiAtlasImage {
    pub texture_atlas: Handle<TextureAtlas>,
    pub index: usize,
}

fn populate_ui_atlas_image(
    mut commands: Commands,
    mut map: ResMut<UiAtlasImageMap>,
    ui_atlas_images: Query<(Entity, &UiAtlasImage), Changed<UiAtlasImage>>,
    mut images: ResMut<Assets<Image>>,
    atlases: Res<Assets<TextureAtlas>>,
) {
    for (entity, ui_atlas_image) in &ui_atlas_images {
        let images = map
            .entry(ui_atlas_image.texture_atlas.clone())
            .or_insert_with(|| {
                let atlas = atlases.get(&ui_atlas_image.texture_atlas).unwrap();

                let image = images
                    .get(&atlas.texture)
                    .unwrap()
                    .clone()
                    .try_into_dynamic()
                    .unwrap(); // TODO: can we handle these errors better?

                atlas
                    .textures
                    .iter()
                    .map(|rect| {
                        let crop = Image::from_dynamic(
                            image.crop_imm(
                                rect.min.x as u32,
                                rect.min.y as u32,
                                rect.width() as u32,
                                rect.height() as u32,
                            ),
                            false, // TODO: what should this be?
                        );

                        images.add(crop)
                    })
                    .collect()
            });

        commands
            .entity(entity)
            .insert(UiImage(images[ui_atlas_image.index].clone()));
    }
}
