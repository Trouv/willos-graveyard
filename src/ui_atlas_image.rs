#![allow(dead_code)]
//! Plugin for displaying images in the UI from a TextureAtlas.
use bevy::prelude::*;
use std::collections::HashMap;

/// Plugin for displaying images in the UI from a TextureAtlas.
///
/// Use the [UiAtlasImage] component to employ this plugin.
pub struct UiAtlasImagePlugin;

impl Plugin for UiAtlasImagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiAtlasImageMap>()
            .add_system(resolve_ui_atlas_image);
    }
}

/// Resource that caches TextureAtlases and their corresponding images.
#[derive(Debug, Default, Deref, DerefMut, Resource)]
struct UiAtlasImageMap(HashMap<Handle<TextureAtlas>, Vec<Handle<Image>>>);

/// Component that defines a UiAtlasImage.
///
/// The plugin will respond to changes in this component.
/// First, it generates plain [Image](bevy::render::Image)s based off the textures in the texture atlas.
/// Using these images, it will insert the appropriate [UiImage](bevy::render::UiImage) on your entity.
#[derive(Clone, Debug, Default, Component)]
pub struct UiAtlasImage {
    /// Atlas that defines the texture and its partitions.
    pub texture_atlas: Handle<TextureAtlas>,
    /// Index of the texture partition to display on this entity.
    pub index: usize,
}

#[derive(Debug, Default, Bundle)]
pub struct AtlasImageBundle {
    pub image_bundle: ImageBundle,
    pub atlas_image: UiAtlasImage,
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

#[cfg(test)]
mod tests {
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    use super::*;
    fn app_setup() -> App {
        let mut app = App::new();

        app.add_plugin(UiAtlasImagePlugin)
            .add_plugin(AssetPlugin::default())
            .add_asset::<Image>()
            .add_asset::<TextureAtlas>();
        app
    }

    fn generate_texture_atlas(app: &mut App) -> Handle<TextureAtlas> {
        let image = Image::new(
            Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![
                0, 0, 100, 100, 0, 0, 100, 100, 200, 200, 255, 255, 200, 200, 255, 255,
            ],
            TextureFormat::R8Unorm,
        );

        let image_handle = app
            .world
            .get_resource_mut::<Assets<Image>>()
            .unwrap()
            .add(image);

        let texture_atlas =
            TextureAtlas::from_grid(image_handle, Vec2::new(2., 2.), 2, 2, None, None);

        app.world
            .get_resource_mut::<Assets<TextureAtlas>>()
            .unwrap()
            .add(texture_atlas)
    }

    #[test]
    fn map_and_image_resolve() {
        let mut app = app_setup();
        let texture_atlas = generate_texture_atlas(&mut app);

        let ui_atlas_image_entity = app
            .world
            .spawn(UiAtlasImage {
                texture_atlas: texture_atlas.clone(),
                index: 1,
            })
            .id();

        app.update();

        let image_handles = app
            .world
            .get_resource::<UiAtlasImageMap>()
            .unwrap()
            .get(&texture_atlas)
            .unwrap();

        let image_assets = app.world.get_resource::<Assets<Image>>().unwrap();

        let images: Vec<&Image> = image_handles
            .iter()
            .map(|h| image_assets.get(h).unwrap())
            .collect();

        // Test that each image contains the right data
        assert_eq!(images[0].data, [0, 0, 0, 255].repeat(4));
        assert_eq!(images[1].data, [100, 100, 100, 255].repeat(4));
        assert_eq!(images[2].data, [200, 200, 200, 255].repeat(4));
        assert_eq!(images[3].data, [255, 255, 255, 255].repeat(4));

        // Test that the entity refers to the correct handle (whose data has already been verified)
        assert_eq!(
            app.world
                .entity(ui_atlas_image_entity)
                .get::<UiImage>()
                .unwrap()
                .0,
            image_handles[1]
        );
    }
}
