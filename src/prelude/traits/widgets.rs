use bevy::prelude::*;

use super::spawn::Spawn;

/// An extension trait for spawning UI widgets.
pub trait Widgets {
    /// Spawn a simple text label.
    fn label(&mut self, text: impl Into<String>) -> EntityCommands;
}

impl<T: Spawn> Widgets for T {
    fn label(&mut self, text: impl Into<String>) -> EntityCommands {
        let entity = self.spawn((
            Name::new("Label"),
            Text(text.into()),
            TextFont::from_font_size(24.0),
            TextColor(Color::srgb(0.867, 0.827, 0.412)),
            Node {
                width: Val::Px(500.0),
                ..default()
            },
        ));
        entity
    }
}
