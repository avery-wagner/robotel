use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
    QueueableCommand,
};

use std::{
    io::{stdout, Stdout, Write},
    thread,
    time::{Duration, Instant},
};

use std::f64::consts::PI as pi;

mod robot;
mod world;

use crate::robot::*;
use crate::world::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut run = true;

    let mut stdout = stdout();
    let mut curr_cmd = Command::Stop;

    let mut robot = Robot::new(Pose::new(5.0, 5.0, pi/2.0), BoundingBox::new(2.0, 2.0), Some(curr_cmd));
    let obj1 = Object::new(Pose::new(10.0, 0.0, 0.0), BoundingBox::new(2.0, 2.0));
    let mut world = World::init(WorldConfig::new(20, 20), robot, vec![obj1], stdout);

    // enable_raw_mode().unwrap();

    // execute!(
    //     world.stdout,
    //     PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    // )?;
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
                    Some(cmd) => curr_cmd = cmd,
                    None => curr_cmd = Command::Stop,
                }
            }
        }

        let dt = now.elapsed().as_secs_f64();
        world.render();
        world.step(dt, curr_cmd);

        // world
        //     .stdout
        //     .queue(crossterm::terminal::Clear(
        //         crossterm::terminal::ClearType::All,
        //     ))
        //     .unwrap();

        // world
        //     .stdout
        //     .queue(crossterm::cursor::MoveTo(10, 10))
        //     .unwrap();
        // write!(world.stdout, "0").unwrap();
        // world.stdout.flush().unwrap();
    }

    // execute!(world.stdout, PopKeyboardEnhancementFlags)?;
    // disable_raw_mode()?;
    world.reset_ui();
    Ok(())
}
