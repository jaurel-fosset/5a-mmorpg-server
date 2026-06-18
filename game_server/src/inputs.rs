use std::collections::HashMap;
use bevy::log::tracing::field::display;
use bevy::prelude::*;
use network_serialization::input::{DirectionFlags, InputData};

struct InputPlugin;

impl Plugin for InputPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .insert_resource(InputStore::new())
            .add_systems(Update, Self::apply_input);
    }
}

impl InputPlugin
{
    fn apply_input(mut inputs_store: ResMut<InputStore>, mut clients: Query<(&Client, &mut Transform)>)
    {
        for (client, mut transform) in clients.iter_mut()
        {
            let inputs = *inputs_store.current_input.get(&client.id).unwrap();
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
                        transform.translation.y -= 10.0;
                    }
                    if input.input.contains(DirectionFlags::DOWN)
                    {
                        transform.translation.y += 10.0;
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
        }

        inputs_store.current_input.clear();
    }
}

#[derive(Resource)]
pub struct InputStore
{
    current_input: HashMap<u32, [InputData; 16]>,
    last_input_sequence: HashMap<u32, u32>,
}

impl InputStore
{
    pub fn new() -> Self
    {
        Self
        {
            current_input: HashMap::new(),
            last_input_sequence: HashMap::new(),
        }
    }
    
    pub fn add_input(&mut self, id: u32, input_data: [InputData; 16])
    {
        self.current_input.insert(id, input_data);
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
