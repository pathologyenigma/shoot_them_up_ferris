// rust way to use the code from others
use bevy::prelude::*;

fn main() {
    App::new()
    .insert_resource(WindowDescriptor{
        title: "shoot them up ferris".to_string(),
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .run();
}
