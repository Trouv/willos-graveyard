use bevy::{prelude::*, window::WindowResized};
use iyes_loopless::prelude::*;

#[derive(SystemLabel)]
pub struct FontScaleLabel;

pub struct FontScalePlugin;

impl Plugin for FontScalePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FontSizeRatios>()
            .add_system(
                font_scale
                    .run_on_event::<WindowResized>()
                    .label(FontScaleLabel),
            )
            .add_system(font_scale.run_if(font_scale_changed).label(FontScaleLabel));
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FontSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Custom(f32),
}

#[derive(Clone, PartialEq, Debug, Default, Component, Deref, DerefMut)]
pub struct FontScale(Vec<FontSize>);

impl From<FontSize> for FontScale {
    fn from(value: FontSize) -> Self {
        FontScale(vec![value])
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct FontSizeRatios {
    pub tiny: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
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
    windows: Res<Windows>,
    ratios: Res<FontSizeRatios>,
) {
    for (font_scale, mut text) in query.iter_mut() {
        if let Some(primary) = windows.get_primary() {
            // To best support ultra-wide and vertical windows, we base the fonts off the smaller
            // of the two dimensions
            let min_length = primary.width().min(primary.height());

            font_scale
                .iter()
                .cycle()
                .zip(text.sections.iter_mut())
                .for_each(|(font_size, mut section)| {
                    section.style.font_size = ratios.get(&font_size) * min_length;
                });
        }
    }
}

fn font_scale_changed(changed_font_scales: Query<Entity, Changed<FontScale>>) -> bool {
    !changed_font_scales.is_empty()
}
