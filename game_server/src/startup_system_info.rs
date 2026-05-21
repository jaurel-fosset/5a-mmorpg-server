use bevy::prelude::*;
use bevy::diagnostic;

pub struct StartupSystemInfoPlugin;

impl Plugin for StartupSystemInfoPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .add_systems(Startup, Self::startup_log);
    }
}

impl StartupSystemInfoPlugin
{
    fn startup_log(sys_info: Option<Res<diagnostic::SystemInfo>>)
    {
        let sys_info = match sys_info
        {
            Some(sys_info) => sys_info,
            None => return,
        };

        info!("=== System Information ===");
        info!("- OS : {}", sys_info.os);
        info!("- CPU : {}", sys_info.cpu);
        info!("- RAM : {}", sys_info.memory);
    }
}