use amethyst::{
    core::{frame_limiter::FrameRateLimitStrategy, Time},
    ecs::{Read, System, Write},
    network::simulation::{
        laminar::{LaminarNetworkBundle, LaminarSocket},
        DeliveryRequirement, NetworkSimulationTime, TransportResource, UrgencyRequirement,
    },
    prelude::*,
    utils::application_root_dir,
    Result,
};
use log::info;
use std::time::Duration;

fn main() -> Result<()> {
    amethyst::start_logger(Default::default());

    let assets_dir = application_root_dir()?.join("./");

    // Laminar
    let socket = LaminarSocket::bind("0.0.0.0:3455")?;

    let game_data = GameDataBuilder::default()
        // Laminar
        .with_bundle(LaminarNetworkBundle::new(Some(socket)))?
        .with(SpamSystem::new(), "spam", &[]);
    let mut game = Application::build(assets_dir, GameState)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .build(game_data)?;
    game.run();
    Ok(())
}
/// Default empty state
pub struct GameState;
impl SimpleState for GameState {}

/// A simple system that sends a ton of messages to all connections.
/// In this case, only the server is connected.
struct SpamSystem {
    last_sent_frame: u32,
}

impl SpamSystem {
    pub fn new() -> Self {
        SpamSystem { last_sent_frame: 0 }
    }
}

impl<'a> System<'a> for SpamSystem {
    type SystemData = (
        Read<'a, NetworkSimulationTime>,
        Read<'a, Time>,
        Write<'a, TransportResource>,
    );
    fn run(&mut self, (sim_time, time, mut net): Self::SystemData) {
        // Use method `sim_time.sim_frames_to_run()` to determine if the system should send a
        // message this frame. If, for example, the ECS frame rate is slower than the simulation
        // frame rate, this code block will run until it catches up with the expected simulation
        // frame number.
        let server_addr = "127.0.0.1:3457".parse().unwrap();
        for frame in sim_time.sim_frames_to_run() {
            if self.last_sent_frame + 60 > frame {
                continue;
            }
            self.last_sent_frame = frame;
            info!("Sending message for sim frame {}.", frame);
            let payload = format!(
                "CL: sim_frame:{},abs_time:{}",
                frame,
                time.absolute_time_seconds()
            );
            net.send_with_requirements(
                server_addr,
                payload.as_bytes(),
                DeliveryRequirement::Reliable,
                UrgencyRequirement::Immediate,
            );
        }
    }
}
