use std::collections::HashMap;
use std::time::Duration;
use bevy::log::tracing::field::display;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use network_serialization::input::{DirectionFlags, InputData};
use crate::client;
use crate::client::NotAuthoritative;

#[derive(Resource)]
struct InputTimer(Timer);

impl Default for InputTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(66), TimerMode::Repeating))
    }
}


pub struct InputPlugin;

impl Plugin for InputPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .init_resource::<InputTimer>()
            .insert_resource(InputStore::new())
            .add_systems(Update, (Self::apply_input, Self::clean_up_clients));
    }
}

impl InputPlugin
{
    fn apply_input(
        time: Res<Time>,
        mut timer: ResMut<InputTimer>,
        mut inputs_store: ResMut<InputStore>,
        mut clients: Query<(&Client, &mut Transform), Without<NotAuthoritative>>
    )
    {
        if !timer.0.tick(time.delta()).just_finished() {
            return;
        }

        for (client, mut transform) in clients.iter_mut()
        {
            let inputs = match inputs_store.current_input.get(&client.id) {
                Some(inputs) => *inputs,
                None => continue,
            };

            let last_sequence = inputs_store.last_input_sequence.entry(client.id)
                .or_insert(0);


            for input in inputs.iter()
            {
                if input.sequence > *last_sequence
                {
                    *last_sequence = input.sequence;

                    if input.input.is_empty() { continue; }

                    if input.input.contains(DirectionFlags::UP)
                    {
                        transform.translation.y += 10.0;
                    }
                    if input.input.contains(DirectionFlags::DOWN)
                    {
                        transform.translation.y -= 10.0;
                    }
                    if input.input.contains(DirectionFlags::LEFT)
                    {
                        transform.translation.x -= 10.0;
                    }
                    if input.input.contains(DirectionFlags::RIGHT)
                    {
                        transform.translation.x += 10.0;
                    }
                }
            }

            info!("Client {} position : x = {}, y = {}", client.id, transform.translation.x, transform.translation.y);
            println!("Client {} position : x = {}, y = {}", client.id, transform.translation.x, transform.translation.y);
        }

        inputs_store.current_input.clear();
    }

    fn clean_up_clients(mut commands: Commands, mut input_store: ResMut<InputStore>, clients: Query<(Entity, &Client), With<NotAuthoritative>>)
    {
        for (entity, client) in clients.iter()
        {
            if let Some(_) = input_store.current_input.get(&client.id)
            {
                input_store.ticks_without_input.insert(client.id, 0);
                continue;
            }

            let ticks = input_store.ticks_without_input.entry(client.id).or_insert(0);
            *ticks += 1;

            if *ticks > 16
            {
                commands.entity(entity).despawn();
                input_store.connected_clients.remove(&client.id);
                input_store.current_input.remove(&client.id);
                input_store.last_input_sequence.remove(&client.id);
                input_store.ticks_without_input.remove(&client.id);
            }
        }
    }
}

#[derive(Resource)]
pub struct InputStore
{
    connected_clients: HashSet<u32>,
    current_input: HashMap<u32, [InputData; 16]>,
    last_input_sequence: HashMap<u32, u32>,
    ticks_without_input: HashMap<u32, u32>,
}

impl InputStore
{
    pub fn new() -> Self
    {
        Self
        {
            connected_clients: HashSet::new(),
            current_input: HashMap::new(),
            last_input_sequence: HashMap::new(),
            ticks_without_input: HashMap::new(),
        }
    }
    
    pub fn add_input(&mut self, id: u32, input_data: [InputData; 16])
    {
        self.connected_clients.insert(id);
        self.current_input.insert(id, input_data);
    }

    pub fn contains_client(&self, id: u32) -> bool
    {
        self.connected_clients.contains(&id)
    }
}

#[derive(Component, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Client
{
    id: u32
}

impl Client
{
    pub fn new(id: u32) -> Self { Self { id } }
    
    pub fn id(self) -> u32 { self.id }
}
