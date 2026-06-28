use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, SetSize},
    ExecutableCommand, QueueableCommand,
};

use std::{
    io::{stdout, Stdout, Write},
    time::Duration,
};

use crate::robot::*;
use std::f64::consts::PI as pi;

// SCALE_X/SCALE_y columns per world x/y-unit => renders terminal square
const SCALE_X: u16 = 2;
const SCALE_Y: u16 = 1;

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

        // TODO check collisions
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
        const MOVE_SPEED: f64 = 1.5;
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
        }
    }

    pub fn setup_ui(&mut self) {
        enable_raw_mode().unwrap();
        self.stdout
            .execute(PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
            ))
            .unwrap();
        // note: execute!(...) -> write to terminal + immediately flush, need for immediate effect of keyboard enhancement flags

        let (sw, sh) = self.screen_dims();
        self.stdout
            .queue(SetSize(sw + 2, sh + 2)) // +2 for the border on each axis
            .unwrap()
            .queue(Clear(ClearType::All))
            .unwrap()
            .queue(Hide)
            .unwrap();

        self.stdout.flush().unwrap()
    }

    pub fn reset_ui(&mut self) {
        let (sw, sh) = self.screen_dims();
        self.stdout
            .queue(SetSize(sw + 2, sh + 2)) // + 2 for the border on each axis
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
        self.draw_objects();
        self.stdout.flush().unwrap();
    }

    // screen size of the simulation terminal cells.
    fn screen_dims(&self) -> (u16, u16) {
        (self.config.w * SCALE_X, self.config.h * SCALE_Y)
    }

    // world point (x right, y UP) → screen cell (col, row)
    // - y-flip (world y-up vs screen rows going down)
    // - aspect scaling
    // - + 1 offset shifts past the border so (0,0) is bottom-left of the interior zone
    fn world_to_screen(&self, x: f64, y: f64) -> (u16, u16) {
        let col = (x * SCALE_X as f64) as u16 + 1;
        let row = ((self.config.h as f64 - y) * SCALE_Y as f64) as u16 + 1;
        (col, row)
    }

    // hollow frame drawn around some w×h interior; the frame itself occupies (w+2)×(h+2) cells.
    // used for the map border.
    fn draw_frame(&mut self, w: u16, h: u16, x0: u16, y0: u16) {
        let corner = ["┌", "┐", "└", "┘"];
        let edge = ["─", "│"];

        let right = x0 + w + 1;
        let bottom = y0 + h + 1;

        // left + right edges
        for y in y0..=bottom {
            self.stdout
                .queue(MoveTo(x0, y))
                .unwrap()
                .queue(Print(edge[1]))
                .unwrap()
                .queue(MoveTo(right, y))
                .unwrap()
                .queue(Print(edge[1]))
                .unwrap();
        }

        // top + bottom edges
        for x in x0..=right {
            self.stdout
                .queue(MoveTo(x, y0))
                .unwrap()
                .queue(Print(edge[0]))
                .unwrap()
                .queue(MoveTo(x, bottom))
                .unwrap()
                .queue(Print(edge[0]))
                .unwrap();
        }

        // corners overwrite edge chars at each junction
        self.stdout
            .queue(MoveTo(x0, y0))
            .unwrap()
            .queue(Print(corner[0]))
            .unwrap()
            .queue(MoveTo(right, y0))
            .unwrap()
            .queue(Print(corner[1]))
            .unwrap()
            .queue(MoveTo(x0, bottom))
            .unwrap()
            .queue(Print(corner[2]))
            .unwrap()
            .queue(MoveTo(right, bottom))
            .unwrap()
            .queue(Print(corner[3]))
            .unwrap();
    }

    fn fill_rect(&mut self, w: u16, h: u16, x0: u16, y0: u16, ch: &str) {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                self.stdout
                    .queue(MoveTo(x, y))
                    .unwrap()
                    .queue(Print(ch))
                    .unwrap();
            }
        }
    }

    fn draw_map(&mut self) {
        let (sw, sh) = self.screen_dims();

        // ---- BORDER ----
        self.stdout
            .queue(SetForegroundColor(Color::DarkGrey))
            .unwrap();
        self.draw_frame(sw, sh, 0, 0);

        // ---- BACKGROUND ----
        // interior cells start at (1, 1)
        self.stdout.queue(ResetColor).unwrap();
        self.fill_rect(sw, sh, 1, 1, " ");
    }

    fn draw_robot(&mut self) {
        self.stdout.queue(SetForegroundColor(Color::Blue)).unwrap();

        let (x0, y0) = self.world_to_screen(self.robot.pose.x, self.robot.pose.y);
        let w = self.robot.bounds.w as u16 * SCALE_X;
        let h = self.robot.bounds.h as u16 * SCALE_Y;

        self.fill_rect(w, h, x0, y0, "█");

        // heading arrow: theta normalized to [0, 2pi) and quantized to 8 dirs,
        // drawn over the robot's center cell
        self.robot.pose.theta = self.robot.pose.theta.rem_euclid(2.0 * pi);
        let i = (self.robot.pose.theta / (2.0 * pi / 8.0)).round() as usize % 8;
        let dir_chars = [">", "↗", "^", "↖", "<", "↙", "v", "↘"];

        let cx = x0 + w / 2;
        let cy = y0 + h / 2;
        self.stdout
            .queue(MoveTo(cx - 2, cy - 2))
            .unwrap()
            .queue(Print(dir_chars[i]))
            .unwrap();
    }

    fn draw_objects(&mut self) {
        self.stdout.queue(SetForegroundColor(Color::Red)).unwrap();

        // collect owned (pose, bounds) out of self.objects BEFORE the loop so the borrow ends and we can call &mut self methods (fill_rect) inside it
        let objects: Vec<_> = self.objects.iter().map(|o| (o.pose, o.bounds)).collect();
        let misc = ["·", "•", "×", "○", "◉", "■", "□"];

        for (pose, bounds) in objects {
            let (x0, y0) = self.world_to_screen(pose.x, pose.y);

            if bounds.w <= 1.0 && bounds.h <= 1.0 {
                // small object → single point
                self.stdout
                    .queue(MoveTo(x0, y0))
                    .unwrap()
                    .queue(Print(misc[5]))
                    .unwrap();
            } else {
                self.fill_rect(
                    bounds.w as u16 * SCALE_X,
                    bounds.h as u16 * SCALE_Y,
                    x0,
                    y0,
                    "█",
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantization() {
        let robot = Robot::new(
            Pose::new(5.0, 5.0, pi / 2.0),
            BoundingBox::new(2.0, 2.0),
            Some(Command::Turn { omega: pi / 2.0 }),
        );
        let obj1 = Object::new(Pose::new(10.0, 0.0, 0.0), BoundingBox::new(2.0, 2.0));
        let mut world = World::init(WorldConfig::new(20, 20), robot, vec![obj1], stdout());

        // 1. normalize theta to fall within [0, 2pi]
        world.robot.pose.theta = world.robot.pose.theta.rem_euclid(2.0 * pi);

        // 2. quantize for 8 directions:
        let i = (world.robot.pose.theta / (2.0 * pi / 8.0)).round() as usize % 8;
        let chars = [">", "↗", "^", "↖", "<", "↙", "v", "↘"];
        let c = chars[i];

        assert!(c == "^");
    }
}
