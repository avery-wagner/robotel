use std::f64::consts::PI as pi;
use std::{io::stdout, time::Instant};

mod robot;
mod world;

use crate::robot::*;
use crate::world::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut run = true;

    let stdout = stdout();
    let mut curr_cmd = Command::Stop;

    let robot = Robot::new(
        Pose::new(5.0, 5.0, pi / 2.0),
        BoundingBox::new(2.0, 2.0),
        Some(curr_cmd),
    );
    let obj1 = Object::new(Pose::new(10.0, 0.0, 0.0), BoundingBox::new(2.0, 2.0));
    let mut world = World::init(WorldConfig::new(20, 20), robot, vec![obj1], stdout);

    world.setup_ui();

    while run {
        let now = Instant::now();

        while now.elapsed() < world.config.timestep {
            if let Some(cmd) = world.get_cmd(world.config.timestep.saturating_sub(now.elapsed())) {
                match Some(cmd) {
                    Some(Command::Quit) => {
                        run = false;
                        break;
                    }
                    _ => curr_cmd = cmd,
                }
            }
        }

        let dt = now.elapsed().as_secs_f64();
        world.render();
        world.step(dt, curr_cmd);
    }
    world.reset_ui();
    Ok(())
}
