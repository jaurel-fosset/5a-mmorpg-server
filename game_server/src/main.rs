use bevy::diagnostic::{DiagnosticsPlugin, SystemInformationDiagnosticsPlugin};
use bevy::prelude::*;
use game_server::env_parameter::EnvParameterPlugin;

fn main()
{
    App::new()
        .add_plugins(DiagnosticsPlugin)
        .add_plugins(SystemInformationDiagnosticsPlugin)
        .add_plugins(EnvParameterPlugin)
        .run();
}
