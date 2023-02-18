use bevy::prelude::*;
use std::collections::HashMap;

pub struct UiAtlasImagePlugin;

impl Plugin for UiAtlasImagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiAtlasImageMap>()
            .add_system(resolve_ui_atlas_image);
    }
}

#[derive(Debug, Default, Deref, DerefMut, Resource)]
struct UiAtlasImageMap(HashMap<Handle<TextureAtlas>, Vec<Handle<Image>>>);

#[derive(Clone, Debug, Default, Component)]
pub struct UiAtlasImage {
    pub texture_atlas: Handle<TextureAtlas>,
    pub index: usize,
}

fn resolve_ui_atlas_image(
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
                let atlas = atlases
                    .get(&ui_atlas_image.texture_atlas)
                    .expect("Handle used in UiAtlasImage should be in Assets<TextureAtlas>");

                let image = images
                    .get(&atlas.texture)
                    .expect("source image for UiAtlasImage should be in Assets<Image>");

                let is_srgb = image.texture_descriptor.format.describe().srgb;

                let dynamic_image = image.clone().try_into_dynamic().expect("source image for UiAtlasImage should support dynamic conversion: https://docs.rs/bevy/latest/bevy/render/texture/struct.Image.html#method.try_into_dynamic");

                atlas
                    .textures
                    .iter()
                    .map(|rect| {
                        let crop = Image::from_dynamic(
                            dynamic_image.crop_imm(
                                rect.min.x as u32,
                                rect.min.y as u32,
                                rect.width() as u32,
                                rect.height() as u32,
                            ),
                            is_srgb,
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
