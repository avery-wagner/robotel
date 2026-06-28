use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType, SetSize},
    ExecutableCommand, QueueableCommand,
};

use std::{
    io::{Stdout, Write},
    thread,
    time::{Duration, Instant},
};

use std::f64::consts::PI as pi;

use crate::robot::*;

#[derive(Debug)]
pub struct WorldConfig {
    w: u16,                 // px
    h: u16,                 // px
    pub timestep: Duration, // s
}

impl WorldConfig {
    pub fn new(w: u16, h: u16) -> Self {
        Self {
            w,
            h,
            timestep: Duration::from_secs_f64(0.05),
        }
    }
}

pub struct World {
    pub config: WorldConfig,
    robot: Robot,
    objects: Vec<Object>,
    clock: f64,
    pub stdout: Stdout,
}

impl World {
    pub fn init(config: WorldConfig, robot: Robot, objects: Vec<Object>, stdout: Stdout) -> Self {
        Self {
            config,
            robot,
            objects,
            clock: 0.0,
            stdout,
        }
    }

    pub fn step(&mut self, dt: f64, cmd: Command) {
        /* advance the world by one timestep */
        self.robot.cmd = Some(cmd);
        self.robot.exec_cmd(dt);

        // check collisions
        // todo!();

        self.clock += dt;
    }

    /* ref: https://github.com/jrhenderson1988/snake-rs/blob/master/src/game.rs */
    fn on_key_event(&self, elapsed: Duration) -> Option<KeyEvent> {
        if poll(elapsed).ok()? {
            let event = read().ok()?;
            if let Event::Key(key_event) = event {
                return Some(key_event);
            }
        }
        None
    }

    pub fn get_cmd(&self, elapsed: Duration) -> Option<Command> {
        /* handle actual direction of turning */

        // TODO probs should move these constants somewhere else
        const TURN_SPEED: f64 = 1.0;
        const MOVE_SPEED: f64 = 1.0;
        let key_event = self.on_key_event(elapsed)?;

        match (key_event.kind, key_event.code) {
            (_, KeyCode::Esc) => Some(Command::Quit), // always quit on 'Esc'
            (_, KeyCode::Char('c') | KeyCode::Char('C')) => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    Some(Command::Quit)
                } else {
                    None
                }
            }
            (
                KeyEventKind::Release,
                KeyCode::Up | KeyCode::Down | KeyCode::Right | KeyCode::Left,
            ) => Some(Command::Stop),
            (KeyEventKind::Press | KeyEventKind::Repeat, KeyCode::Up) => {
                Some(Command::Move { v: MOVE_SPEED })
            }
            (KeyEventKind::Press | KeyEventKind::Repeat, KeyCode::Down) => {
                Some(Command::Move { v: -MOVE_SPEED })
            }
            (KeyEventKind::Press | KeyEventKind::Repeat, KeyCode::Right) => {
                Some(Command::Turn { omega: -TURN_SPEED }) // CW
            }
            (KeyEventKind::Press | KeyEventKind::Repeat, KeyCode::Left) => {
                Some(Command::Turn { omega: TURN_SPEED }) // CCW
            }
            _ => Some(Command::Stop),
            // _ => None,
        }
    }

    pub fn setup_ui(&mut self) {
        enable_raw_mode().unwrap();
        self.stdout
            .execute(PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
            ))
            .unwrap(); // execute!(...) -> write to terminal + immediately flush, need for immediate effect of keyboard enhancement flags

        self.stdout
            .queue(SetSize(self.config.w, self.config.h))
            .unwrap()
            .queue(Clear(ClearType::All))
            .unwrap()
            .queue(Hide)
            .unwrap();

        self.stdout.flush().unwrap()
    }

    pub fn reset_ui(&mut self) {
        self.stdout
            .queue(SetSize(self.config.w, self.config.h))
            .unwrap()
            .queue(Clear(ClearType::All))
            .unwrap()
            .queue(Show)
            .unwrap()
            .queue(ResetColor)
            .unwrap();

        self.stdout.execute(PopKeyboardEnhancementFlags).unwrap();
        disable_raw_mode().unwrap();
    }

    pub fn render(&mut self) {
        self.draw_map();
        self.draw_robot();
        // self.draw_objects();
        self.stdout.flush().unwrap();
    }

    fn draw_map(&mut self) {
        // ---- BORDERS ----
        self.stdout
            .queue(SetForegroundColor(Color::DarkGrey))
            .unwrap();

        for y in 0..self.config.h + 2 {
            self.stdout
                .queue(MoveTo(0, y))
                .unwrap()
                .queue(Print("#"))
                .unwrap()
                .queue(MoveTo(self.config.w + 1, y))
                .unwrap()
                .queue(Print("#"))
                .unwrap();
        }

        for x in 0..self.config.w + 2 {
            self.stdout
                .queue(MoveTo(x, 0))
                .unwrap()
                .queue(Print("#"))
                .unwrap()
                .queue(MoveTo(x, self.config.h + 1))
                .unwrap()
                .queue(Print("#"))
                .unwrap();
        }

        self.stdout
            .queue(MoveTo(0, 0))
            .unwrap()
            .queue(Print("#"))
            .unwrap()
            .queue(MoveTo(self.config.w + 1, self.config.h + 1))
            .unwrap()
            .queue(Print("#"))
            .unwrap()
            .queue(MoveTo(self.config.w + 1, 0))
            .unwrap()
            .queue(Print("#"))
            .unwrap()
            .queue(MoveTo(0, self.config.h + 1))
            .unwrap()
            .queue(Print("#"))
            .unwrap();

        // ---- BACKGROUND ----
        self.stdout.queue(ResetColor).unwrap();

        for y in 1..self.config.h + 1 {
            for x in 1..self.config.w + 1 {
                self.stdout
                    .queue(MoveTo(x, y))
                    .unwrap()
                    .queue(Print(" "))
                    .unwrap();
            }
        }
    }

    fn draw_robot(&mut self) {
        self.stdout
            .queue(SetForegroundColor(Color::DarkBlue))
            .unwrap();

        // 1. normalize theta to fall within [0, 2pi]
        self.robot.pose.theta = self.robot.pose.theta.rem_euclid(2.0 * pi);

        // 2. quantize for 8 directions:
        let i = (self.robot.pose.theta / (2.0 * pi / 8.0)).round() as usize % 8;
        // let chars = ["<", "↙", "v", "↘", ">", "↗", "^", "↖"];
        let chars = [">", "↗", "^", "↖", "<", "↙", "v", "↘"];

        let c = chars[i];

        let x = self.robot.pose.x as u16 + 1;
        // let y = self.robot.pose.y as u16 + 1;
        let y = (self.config.h as f64 - self.robot.pose.y) as u16 + 1;

        self.stdout
            .queue(MoveTo(x, y))
            .unwrap()
            .queue(Print(c))
            .unwrap();
    }

    fn draw_objects(&mut self) {
        todo!()
    }
}
