#![allow(dead_code)]
//! Plugin for displaying images in the UI from a TextureAtlas.
use bevy::{asset::RenderAssetUsages, prelude::*};
use std::collections::HashMap;

/// Plugin for displaying images in the UI from a TextureAtlas.
///
/// Use the [UiAtlasImage] component to employ this plugin.
pub struct UiAtlasImagePlugin;

impl Plugin for UiAtlasImagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiAtlasImageMap>()
            .add_systems(PostUpdate, resolve_ui_atlas_image);
    }
}

/// Resource that caches TextureAtlases and their corresponding images.
#[derive(Debug, Default, Deref, DerefMut, Resource)]
struct UiAtlasImageMap(HashMap<Handle<TextureAtlasLayout>, Vec<Handle<Image>>>);

/// Component that defines a UiAtlasImage.
///
/// The plugin will respond to changes in this component.
/// First, it generates plain [Image](bevy::render::Image)s based off the textures in the texture atlas.
/// Using these images, it will insert the appropriate `ImageNode` on your entity.
#[derive(Clone, Debug, Default, Eq, PartialEq, Component)]
pub struct UiAtlasImage {
    /// Source image.
    pub image: Handle<Image>,
    /// Atlas that defines the texture and its partitions.
    pub texture_atlas: Handle<TextureAtlasLayout>,
    /// Index of the texture partition to display on this entity.
    pub index: usize,
}

fn resolve_ui_atlas_image(
    mut commands: Commands,
    mut map: ResMut<UiAtlasImageMap>,
    ui_atlas_images: Query<(Entity, &UiAtlasImage), Changed<UiAtlasImage>>,
    mut images: ResMut<Assets<Image>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
) {
    for (entity, ui_atlas_image) in &ui_atlas_images {
        let images = map
            .entry(ui_atlas_image.texture_atlas.clone())
            .or_insert_with(|| {
                let atlas = atlases
                    .get(&ui_atlas_image.texture_atlas)
                    .expect("Handle used in UiAtlasImage should be in Assets<TextureAtlas>");

                let image = images
                    .get(&ui_atlas_image.image)
                    .expect("source image for UiAtlasImage should be in Assets<Image>");

                let is_srgb = image.texture_descriptor.format.is_srgb();

                let dynamic_image = image.clone().try_into_dynamic().expect("source image for UiAtlasImage should support dynamic conversion: https://docs.rs/bevy/latest/bevy/render/texture/struct.Image.html#method.try_into_dynamic");

                atlas
                    .textures
                    .iter()
                    .map(|rect| {
                        let crop = Image::from_dynamic(
                            dynamic_image.crop_imm(
                                rect.min.x,
                                rect.min.y,
                                rect.width(),
                                rect.height(),
                            ),
                            is_srgb,
                            RenderAssetUsages::RENDER_WORLD,
                        );

                        images.add(crop)
                    })
                    .collect()
            });

        commands
            .entity(entity)
            .insert(ImageNode::new(images[ui_atlas_image.index].clone()));
    }
}

#[cfg(test)]
mod tests {
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    use super::*;
    fn app_setup() -> App {
        let mut app = App::new();

        app.add_plugins((UiAtlasImagePlugin, AssetPlugin::default()))
            .init_asset::<Image>()
            .init_asset::<TextureAtlasLayout>();
        app
    }

    fn generate_texture_atlas(app: &mut App) -> (Handle<Image>, Handle<TextureAtlasLayout>) {
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
            RenderAssetUsages::MAIN_WORLD,
        );

        let image_handle = app
            .world_mut()
            .get_resource_mut::<Assets<Image>>()
            .unwrap()
            .add(image);

        let texture_atlas = TextureAtlasLayout::from_grid(UVec2::new(2, 2), 2, 2, None, None);

        let texture_atlas_handle = app
            .world_mut()
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .unwrap()
            .add(texture_atlas);

        (image_handle, texture_atlas_handle)
    }

    fn spawn_ui_atlas_image_entity(
        app: &mut App,
        image: Handle<Image>,
        texture_atlas: Handle<TextureAtlasLayout>,
    ) -> Entity {
        app.world_mut()
            .spawn(UiAtlasImage {
                image,
                texture_atlas,
                index: 1,
            })
            .id()
    }

    #[test]
    fn map_and_image_resolve() {
        let mut app = app_setup();
        let (image, texture_atlas) = generate_texture_atlas(&mut app);
        let ui_atlas_image_entity =
            spawn_ui_atlas_image_entity(&mut app, image, texture_atlas.clone());

        app.update();

        let image_handles = app
            .world()
            .get_resource::<UiAtlasImageMap>()
            .unwrap()
            .get(&texture_atlas)
            .unwrap();

        let image_assets = app.world().get_resource::<Assets<Image>>().unwrap();

        let images: Vec<&Image> = image_handles
            .iter()
            .map(|h| image_assets.get(h).unwrap())
            .collect();

        // Test that each image contains the right data
        assert_eq!(images[0].data, Some([0, 0, 0, 255].repeat(4)));
        assert_eq!(images[1].data, Some([100, 100, 100, 255].repeat(4)));
        assert_eq!(images[2].data, Some([200, 200, 200, 255].repeat(4)));
        assert_eq!(images[3].data, Some([255, 255, 255, 255].repeat(4)));

        // Test that the entity's UiImage resolved appropriately (whose data has already been verified)
        assert_eq!(
            app.world()
                .entity(ui_atlas_image_entity)
                .get::<ImageNode>()
                .unwrap()
                .image,
            image_handles[1]
        );
    }

    #[test]
    fn index_changes_dont_generate_more_images() {
        let mut app = app_setup();
        let (image, texture_atlas) = generate_texture_atlas(&mut app);
        let ui_atlas_image_entity =
            spawn_ui_atlas_image_entity(&mut app, image, texture_atlas.clone());

        app.update();

        let image_handles = app
            .world()
            .get_resource::<UiAtlasImageMap>()
            .unwrap()
            .get(&texture_atlas)
            .unwrap()
            .clone();

        // Test that the entity's UiImage resolved appropriately
        assert_eq!(
            app.world()
                .entity(ui_atlas_image_entity)
                .get::<ImageNode>()
                .unwrap()
                .image,
            image_handles[1]
        );

        app.world_mut()
            .entity_mut(ui_atlas_image_entity)
            .get_mut::<UiAtlasImage>()
            .unwrap()
            .index += 1;

        app.update();

        let new_image_handles = app
            .world()
            .get_resource::<UiAtlasImageMap>()
            .unwrap()
            .get(&texture_atlas)
            .unwrap();

        // Test the entity's UiImage *changed* appropriately
        assert_eq!(
            app.world()
                .entity(ui_atlas_image_entity)
                .get::<ImageNode>()
                .unwrap()
                .image,
            image_handles[2]
        );

        // Test that the image handles have not changed
        assert_eq!(image_handles, *new_image_handles);
    }
}
