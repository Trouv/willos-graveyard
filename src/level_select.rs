use crate::AssetHolder;
use bevy::{ecs::system::EntityCommands, prelude::*};

fn spawn_level_select_card<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    asset_holder: &AssetHolder,
) -> EntityCommands<'w, 's, 'a> {
    commands.spawn()
}
