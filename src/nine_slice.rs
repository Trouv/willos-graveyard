use bevy::{
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension},
        texture::TextureFormatPixelInfo,
    },
    sprite::Rect,
};
use thiserror::Error;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Error)]
pub enum NineSliceError {
    #[error("source image not found")]
    ImageNotFound,
}

type Result<T> = std::result::Result<T, NineSliceError>;

/// Sprite sheet indices for a 9-slice object
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct NineSliceIndex {
    /// Index data, defined in the following order:
    /// \[top-left, top, top-right, left, center, right, bottom-left, bottom, bottom-right\]
    pub indices: [usize; 9],
}

impl Default for NineSliceIndex {
    fn default() -> Self {
        NineSliceIndex {
            indices: [0, 1, 2, 3, 4, 5, 6, 7, 8],
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct NineSliceSize {
    /// the width of the 9-slice object in "inner" tiles, excluding the borders
    pub inner_width: u32,
    /// the height of the 9-slice object in "inner" tiles, excluding the borders
    pub inner_height: u32,
}

impl From<NineSliceSize> for UVec2 {
    fn from(nine_slice_size: NineSliceSize) -> Self {
        UVec2::new(nine_slice_size.inner_width, nine_slice_size.inner_height)
    }
}

fn get_pixel(image: &Image, coords: UVec2) -> &[u8] {
    let pixel_size_in_bytes = image.texture_descriptor.format.pixel_size();

    let pixel_start = coords.y as usize * image.size().x as usize * pixel_size_in_bytes
        + coords.x as usize * pixel_size_in_bytes;

    &image.data[pixel_start..(pixel_start + pixel_size_in_bytes)]
}

fn push_nine_slice_row_data(
    buffer: &mut Vec<u8>,
    image: &Image,
    left_rect: Rect,
    middle_rect: Rect,
    right_rect: Rect,
    row_height: u32,
    middle_count: u32,
) {
    for y in 0..row_height {
        for x in 0..left_rect.width() as u32 {
            let coord = left_rect.min.as_uvec2() + UVec2::new(x, y);
            buffer.extend(get_pixel(image, coord));
        }

        for _ in 0..middle_count {
            for x in 0..middle_rect.width() as u32 {
                let coord = middle_rect.min.as_uvec2() + UVec2::new(x, y);
                buffer.extend(get_pixel(image, coord));
            }
        }

        for x in 0..right_rect.width() as u32 {
            let coord = right_rect.min.as_uvec2() + UVec2::new(x, y);
            buffer.extend(get_pixel(image, coord));
        }
    }
}

/// Generate a new image from 9-slice data
pub fn generate_nineslice_image(
    size: NineSliceSize,
    NineSliceIndex { indices }: NineSliceIndex,
    source_atlas: &TextureAtlas,
    images: &mut Assets<Image>,
) -> Result<Handle<Image>> {
    let top_left_rect = source_atlas.textures[indices[0]];
    let top_rect = source_atlas.textures[indices[1]];
    let top_right_rect = source_atlas.textures[indices[2]];
    let left_rect = source_atlas.textures[indices[3]];
    let center_rect = source_atlas.textures[indices[4]];
    let right_rect = source_atlas.textures[indices[5]];
    let bottom_left_rect = source_atlas.textures[indices[6]];
    let bottom_rect = source_atlas.textures[indices[7]];
    let bottom_right_rect = source_atlas.textures[indices[8]];

    let source_image = images
        .get(&source_atlas.texture)
        .ok_or(NineSliceError::ImageNotFound)?;

    let image_size = top_left_rect.size().as_uvec2()
        + (UVec2::from(size) * center_rect.size().as_uvec2())
        + bottom_right_rect.size().as_uvec2();

    let texture_dimension = TextureDimension::D2;

    let texture_format = source_image.texture_descriptor.format;

    let mut data: Vec<u8> = Vec::new();

    push_nine_slice_row_data(
        &mut data,
        source_image,
        top_left_rect,
        top_rect,
        top_right_rect,
        top_left_rect.height() as u32,
        size.inner_width,
    );

    for _ in 0..size.inner_height {
        push_nine_slice_row_data(
            &mut data,
            source_image,
            left_rect,
            center_rect,
            right_rect,
            center_rect.height() as u32,
            size.inner_width,
        );
    }

    push_nine_slice_row_data(
        &mut data,
        source_image,
        bottom_left_rect,
        bottom_rect,
        bottom_right_rect,
        bottom_right_rect.height() as u32,
        size.inner_width,
    );

    let image = Image::new(
        Extent3d {
            width: image_size.x,
            height: image_size.y,
            depth_or_array_layers: 1,
        },
        texture_dimension,
        data,
        texture_format,
    );

    Ok(images.add(image))
}

/// Construct a texture atlas from 9-slice data
///
/// The `left`, `right`, `top` and `bottom` arguments represent the locations of the slices in
/// terms of their distance from the border.
pub fn texture_atlas_from_nine_slice(
    texture: Handle<Image>,
    dimensions: Vec2,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
) -> TextureAtlas {
    let mut texture_atlas = TextureAtlas::new_empty(texture, dimensions);

    texture_atlas.textures.push(Rect {
        min: Vec2::new(0., 0.),
        max: Vec2::new(left, top),
    });
    texture_atlas.textures.push(Rect {
        min: Vec2::new(left, 0.),
        max: Vec2::new(dimensions.x - right, top),
    });
    texture_atlas.textures.push(Rect {
        min: Vec2::new(dimensions.x - right, 0.),
        max: Vec2::new(dimensions.x, top),
    });

    texture_atlas.textures.push(Rect {
        min: Vec2::new(0., top),
        max: Vec2::new(left, dimensions.y - bottom),
    });
    texture_atlas.textures.push(Rect {
        min: Vec2::new(left, top),
        max: Vec2::new(dimensions.x - right, dimensions.y - bottom),
    });
    texture_atlas.textures.push(Rect {
        min: Vec2::new(dimensions.x - right, top),
        max: Vec2::new(dimensions.x, dimensions.y - bottom),
    });

    texture_atlas.textures.push(Rect {
        min: Vec2::new(0., dimensions.y - bottom),
        max: Vec2::new(left, dimensions.y),
    });
    texture_atlas.textures.push(Rect {
        min: Vec2::new(left, dimensions.y - bottom),
        max: Vec2::new(dimensions.x - right, dimensions.y),
    });
    texture_atlas.textures.push(Rect {
        min: Vec2::new(dimensions.x - right, dimensions.y - bottom),
        max: Vec2::new(dimensions.x, dimensions.y),
    });

    texture_atlas
}
