//! Utilities for generating nine-slice images from texture atlases.
use bevy::{
    asset::RenderAssetUsages,
    image::TextureFormatPixelInfo,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension},
};
use thiserror::Error;

/// Errors encountered by the nine-slice API.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Error)]
pub enum NineSliceError {
    /// Source Image for the 9-slice texture atlas not found in `Assets<Image>`.
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

/// Size of a nine-slice image expressed in terms of "inner" tile-count.
///
/// The "inner" tile-counts exclude the border tiles.
/// This type is designed that way so that all possible values of the [u32] fields are valid.
/// For example, even a size of `(0, 0)` will produce a valid image of just border tiles.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct NineSliceSize {
    /// the width of the 9-slice object in "inner" tiles, excluding the borders.
    pub inner_width: u32,
    /// the height of the 9-slice object in "inner" tiles, excluding the borders.
    pub inner_height: u32,
}

impl From<NineSliceSize> for UVec2 {
    fn from(nine_slice_size: NineSliceSize) -> Self {
        UVec2::new(nine_slice_size.inner_width, nine_slice_size.inner_height)
    }
}

fn get_pixel(image: &Image, coords: UVec2) -> Option<&[u8]> {
    let pixel_size_in_bytes = image.texture_descriptor.format.pixel_size();

    let pixel_start = coords.y as usize * image.size().x as usize * pixel_size_in_bytes
        + coords.x as usize * pixel_size_in_bytes;

    image
        .data
        .as_ref()
        .map(|d| &d[pixel_start..(pixel_start + pixel_size_in_bytes)])
}

fn push_nine_slice_row_data(
    buffer: &mut Vec<u8>,
    image: &Image,
    left_rect: URect,
    middle_rect: URect,
    right_rect: URect,
    row_height: u32,
    middle_count: u32,
) {
    for y in 0..row_height {
        for x in 0..left_rect.width() {
            let coord = left_rect.min + UVec2::new(x, y);
            if let Some(pixel) = get_pixel(image, coord) {
                buffer.extend(pixel);
            }
        }

        for _ in 0..middle_count {
            for x in 0..middle_rect.width() {
                let coord = middle_rect.min + UVec2::new(x, y);
                if let Some(pixel) = get_pixel(image, coord) {
                    buffer.extend(pixel);
                }
            }
        }

        for x in 0..right_rect.width() {
            let coord = right_rect.min + UVec2::new(x, y);
            if let Some(pixel) = get_pixel(image, coord) {
                buffer.extend(pixel);
            }
        }
    }
}

/// Generate a new image from 9-slice data.
pub fn generate_nineslice_image(
    size: NineSliceSize,
    NineSliceIndex { indices }: NineSliceIndex,
    source_atlas: &TextureAtlasLayout,
    image_handle: &Handle<Image>,
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
        .get(image_handle)
        .ok_or(NineSliceError::ImageNotFound)?;

    let image_size =
        top_left_rect.size() + (UVec2::from(size) * center_rect.size()) + bottom_right_rect.size();

    let texture_dimension = TextureDimension::D2;

    let texture_format = source_image.texture_descriptor.format;

    let mut data: Vec<u8> = Vec::new();

    push_nine_slice_row_data(
        &mut data,
        source_image,
        top_left_rect,
        top_rect,
        top_right_rect,
        top_left_rect.height(),
        size.inner_width,
    );

    for _ in 0..size.inner_height {
        push_nine_slice_row_data(
            &mut data,
            source_image,
            left_rect,
            center_rect,
            right_rect,
            center_rect.height(),
            size.inner_width,
        );
    }

    push_nine_slice_row_data(
        &mut data,
        source_image,
        bottom_left_rect,
        bottom_rect,
        bottom_right_rect,
        bottom_right_rect.height(),
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
        RenderAssetUsages::all(),
    );

    Ok(images.add(image))
}

/// Construct a texture atlas from 9-slice data.
///
/// The `left`, `right`, `top` and `bottom` arguments represent the locations of the slices in
/// terms of their distance from the border.
pub fn texture_atlas_from_nine_slice(
    dimensions: UVec2,
    left: u32,
    right: u32,
    top: u32,
    bottom: u32,
) -> TextureAtlasLayout {
    let mut texture_atlas = TextureAtlasLayout::new_empty(dimensions);

    texture_atlas.textures.push(URect {
        min: UVec2::new(0, 0),
        max: UVec2::new(left, top),
    });
    texture_atlas.textures.push(URect {
        min: UVec2::new(left, 0),
        max: UVec2::new(dimensions.x - right, top),
    });
    texture_atlas.textures.push(URect {
        min: UVec2::new(dimensions.x - right, 0),
        max: UVec2::new(dimensions.x, top),
    });

    texture_atlas.textures.push(URect {
        min: UVec2::new(0, top),
        max: UVec2::new(left, dimensions.y - bottom),
    });
    texture_atlas.textures.push(URect {
        min: UVec2::new(left, top),
        max: UVec2::new(dimensions.x - right, dimensions.y - bottom),
    });
    texture_atlas.textures.push(URect {
        min: UVec2::new(dimensions.x - right, top),
        max: UVec2::new(dimensions.x, dimensions.y - bottom),
    });

    texture_atlas.textures.push(URect {
        min: UVec2::new(0, dimensions.y - bottom),
        max: UVec2::new(left, dimensions.y),
    });
    texture_atlas.textures.push(URect {
        min: UVec2::new(left, dimensions.y - bottom),
        max: UVec2::new(dimensions.x - right, dimensions.y),
    });
    texture_atlas.textures.push(URect {
        min: UVec2::new(dimensions.x - right, dimensions.y - bottom),
        max: UVec2::new(dimensions.x, dimensions.y),
    });

    texture_atlas
}
