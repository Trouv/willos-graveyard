use bevy::prelude::*;

pub fn make_ui_visible(mut ui_query: Query<&mut Visibility, With<Node>>) {
    for mut visibility in ui_query.iter_mut() {
        visibility.is_visible = true;
    }
}
