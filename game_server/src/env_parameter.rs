use std::env::VarError;
use std::num::ParseIntError;
use std::str::FromStr;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use thiserror::Error;
use crate::schedule_handling::ScheduleFactory;

pub struct EnvParameterPlugin;

impl Plugin for EnvParameterPlugin
{
    fn build(&self, app: &mut App)
    {
        ScheduleFactory::register(app, EnvironmentSetup)
            .before_startup(PreStartup);

        let environment = match Environment::new()
        {
            Ok(environment) => environment,
            Err(error) => panic!("Failed to initialize environment:\n {}", error),
        };

        app
            .insert_resource(environment);
    }
}

#[derive(ScheduleLabel, Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct EnvironmentSetup;

#[derive(Resource)]
pub struct Environment
{
    pub player_capacity: u8,
    pub ip: String,
    pub port: u16,
}

impl Environment
{
    pub fn new() -> Result<Self, EnvironmentSetupError>
    {
        let capacity = Self::get_player_capacity()?;
        let ip = Self::get_ip()?;
        let port = Self::get_port()?;

        Ok(Self
        {
            player_capacity: capacity,
            ip,
            port,
        })
    }

    fn get_player_capacity() -> Result<u8, EnvironmentSetupError>
    {
        let player_capacity = match std::env::var("PLAYER_CAPACITY")
        {
            Ok(player_capacity) => player_capacity,
            Err(error) => return Err(EnvironmentSetupError::from_var_error("PLAYER_CAPACITY", error)),
        };
        let player_capacity = match u8::from_str(player_capacity.as_str())
        {
            Ok(player_capacity) => player_capacity,
            Err(error) =>
            {
                return Err(EnvironmentSetupError::from_parse_int_error("PLAYER_CAPACITY", error));
            }
        };

        Ok(player_capacity)
    }

    fn get_ip() -> Result<String, EnvironmentSetupError>
    {
        if let Ok(ip) = std::env::var("IP") {
            if ip != "0.0.0.0" && ip != "127.0.0.1" {
                return Ok(ip);
            }
        }

        // Détecte l'IP locale réelle
        let socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.connect("8.8.8.8:80").unwrap();
        let ip = socket.local_addr().unwrap().ip().to_string();
        Ok(ip)
    }

    fn get_port() -> Result<u16, EnvironmentSetupError>
    {
        let port = match std::env::var("PORT")
        {
            Ok(port) => port,
            Err(error) => return Err(EnvironmentSetupError::from_var_error("PORT", error)),
        };
        let port = match u16::from_str(port.as_str())
        {
            Ok(port) => port,
            Err(error) => return Err(EnvironmentSetupError::from_parse_int_error("PORT", error)),
        };
        Ok(port)
    }
}

#[derive(Debug, Error)]
pub enum EnvironmentSetupError
{
    #[error("Could not find environment variable {variable_name}")]
    VariableNotFound { variable_name: String },
    #[error("Variable {variable_name} is ill formed because : {ill_form_reason}")]
    VariableIllFormed { variable_name: String, ill_form_reason: String },
}


impl EnvironmentSetupError
{
    pub fn from_var_error(variable_name: &str, error: VarError) -> Self
    {
        let variable_name = variable_name.to_string();
        match error
        {
            VarError::NotPresent => Self::VariableNotFound
            {
                variable_name,
            },
            VarError::NotUnicode(_) => Self::VariableIllFormed
            {
                variable_name,
                ill_form_reason: "Not valid unicode".to_string(),
            },
        }
    }

    pub fn from_parse_int_error(variable_name: &str, error: ParseIntError) -> Self
    {
        let variable_name = variable_name.to_string();
        Self::VariableIllFormed
        {
            variable_name,
            ill_form_reason: format!("could not parse to integer ({})", error),
        }
    }
}
