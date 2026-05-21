use bevy::diagnostic;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::prelude::*;

use game_server::env_parameter::EnvParameterPlugin;
use game_server::network::NetworkPluginGroup;
use game_server::startup_system_info::StartupSystemInfoPlugin;

fn main()
{
    App::new()
        //.add_plugins(DefaultPlugins)
        .add_plugins(MinimalPlugins)
        .add_plugins(DiagnosticsPlugin)
        .add_plugins(diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(EnvParameterPlugin)
        .add_plugins(StartupSystemInfoPlugin)
        .add_plugins(NetworkPluginGroup)
        .run();
}
