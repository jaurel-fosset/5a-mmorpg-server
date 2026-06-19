use std::time::Duration;
use bevy::prelude::*;
use bytes::BytesMut;
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::broker::PublishPacket;
use network_serialization::packets::Packet;
use network_serialization::packets::topic::TopicTree;
use network_serialization::Serializable;
use crate::inputs::Client;
use crate::network::broker::BrokerPeer;

#[derive(Resource)]
struct FrameTimer(Timer);

impl Default for FrameTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(66), TimerMode::Repeating))
    }
}

pub struct FramePlugin;

impl Plugin for FramePlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .init_resource::<FrameTimer>()
            .add_systems(Last, Self::send_frame);
    }
}

impl FramePlugin
{
    fn send_frame(
        time: Res<Time>,
        mut timer: ResMut<FrameTimer>,
        broker: Option<Res<BrokerPeer>>,
        clients: Query<(&Client, &Transform)>
    )
    {
        if !timer.0.tick(time.delta()).just_finished() {
            return;
        }

        let broker = match broker {
            None => return,
            Some(broker) => broker,
        };

        let frame = match Self::make_frame(clients) {
            Some(frame) => frame,
            None => return,
        };

        let packet = PacketMessage::new(PacketData::Publish(PublishPacket {
            data: vec![frame],
        }));
        let bytes = packet.write().unwrap();

        if let Err(_) = broker.send(bytes)
        {
            warn!("Failed to send a frame");
        }
        else
        {
            info!("Sent a frame");
        }
    }

    fn make_frame(clients: Query<(&Client, &Transform)>) -> Option<TopicTree>
    {
        if clients.is_empty() {
            return None;
        }

        let mut tree_entities = TopicTree::new_empty("entities".to_string());
        // Position
        let mut tree_position = TopicTree::new_empty("position".to_string());

        for (client, position) in clients.iter()
        {
            let mut bytes = BytesMut::new();
            let _ = position.translation.x.serialize(&mut bytes);
            let _ = position.translation.y.serialize(&mut bytes);

            tree_position.add_leaf(client.id().to_string(), Vec::<u8>::from(bytes));
        }

        tree_entities.add_tree(tree_position);
        Some(tree_entities)
    }
}