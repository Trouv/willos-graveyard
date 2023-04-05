//! Plugin, systems, components, and resources for scaling fonts with window size.
use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

/// Set used by all systems in [FontScalePlugin].
#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
pub struct FontScaleSet;

/// Plugin with systems and resources that implement [FontScale] functionality.
pub struct FontScalePlugin;

impl Plugin for FontScalePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FontSizeRatios>()
            .add_system(
                font_scale
                    .run_if(on_event::<WindowResized>())
                    .in_set(FontScaleSet),
            )
            .add_system(font_scale.run_if(font_scale_changed).in_set(FontScaleSet));
    }
}

/// Font sizes available for font scaling with the [FontScale] component.
#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FontSize {
    /// A tiny font size, good for footnotes.
    Tiny,
    /// A small font size, good for description bodies.
    Small,
    /// A medium font size, good for button text and sub-titles.
    Medium,
    /// A large font size, good for section titles.
    Large,
    /// A huge font size, good for menu titles.
    Huge,
    /// A custom font size defined by the font size ratio.
    ///
    /// See [FontSizeRatios] for more details on font size ratios.
    Custom(f32),
}

/// Component for making fonts scale with window size.
///
/// You can provide a list of font sizes for text with multiple sections.
/// If the number of font sizes exceeds the number of sections, not all the sizes will be used.
/// If the number of sections exceeds the number of font sizes, the sizes used will be used in a
/// loop.
///
/// As a result, text with multiple sections can have a uniform font size by just using a
/// single-element `FontSize` list.
/// A single-element `FontScale` can be instantiated with `FontScale::from(FontSize::...)`.
#[derive(Clone, PartialEq, Debug, Default, Component, Deref, DerefMut)]
pub struct FontScale(pub Vec<FontSize>);

impl From<FontSize> for FontScale {
    fn from(value: FontSize) -> Self {
        FontScale(vec![value])
    }
}

/// Resource for defining the ratio of [FontSize] options to screen dimensions.
///
/// Each field corresponds to a different [FontSize] variant, and its value is used to scale text
/// using that [FontSize].
///
/// These values can be calculated as the result of `desired_font_size / given_screen_height` for
/// wide screens, and `desired_font_size / given_screen_width` for tall screens.
#[derive(Copy, Clone, PartialEq, Debug, Resource)]
pub struct FontSizeRatios {
    /// Font size ratio corresponding to [FontSize::Tiny]
    pub tiny: f32,
    /// Font size ratio corresponding to [FontSize::Small]
    pub small: f32,
    /// Font size ratio corresponding to [FontSize::Medium]
    pub medium: f32,
    /// Font size ratio corresponding to [FontSize::Large]
    pub large: f32,
    /// Font size ratio corresponding to [FontSize::Huge]
    pub huge: f32,
}

impl FontSizeRatios {
    fn get(&self, font_size: &FontSize) -> f32 {
        match font_size {
            FontSize::Tiny => self.tiny,
            FontSize::Small => self.small,
            FontSize::Medium => self.medium,
            FontSize::Large => self.large,
            FontSize::Huge => self.huge,
            FontSize::Custom(r) => *r,
        }
    }
}

impl Default for FontSizeRatios {
    fn default() -> Self {
        FontSizeRatios {
            tiny: 0.01,
            small: 0.02,
            medium: 0.03,
            large: 0.04,
            huge: 0.05,
        }
    }
}

fn font_scale(
    mut query: Query<(&FontScale, &mut Text)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ratios: Res<FontSizeRatios>,
) {
    for (font_scale, mut text) in query.iter_mut() {
        if let Ok(primary) = windows.get_single() {
            // To best support ultra-wide and vertical windows, we base the fonts off the smaller
            // of the two dimensions
            let min_length = primary.width().min(primary.height());

            font_scale
                .iter()
                .cycle()
                .zip(text.sections.iter_mut())
                .for_each(|(font_size, mut section)| {
                    section.style.font_size = ratios.get(font_size) * min_length;
                });
        }
    }
}

fn font_scale_changed(changed_font_scales: Query<Entity, Changed<FontScale>>) -> bool {
    !changed_font_scales.is_empty()
}
