use std::time::Duration;

#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    w: f64,
    h: f64,
}

impl BoundingBox {
    pub fn new(w: f64, h: f64) -> Self {
        Self { w, h }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Pose {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Move { v: f64 },     // v = velocity
    Turn { omega: f64 }, // omega = ang vel
    Stop,
    Quit
}

#[derive(Debug, Clone, Copy)]
pub struct Robot {
    pub pose: Pose,
    pub bounds: BoundingBox,
    pub cmd: Option<Command>,
}

#[derive(Debug, Clone, Copy)]
pub struct Object {
    pose: Pose,
    bounds: BoundingBox,
}

impl Object {
    pub fn new(pose: Pose, bounds: BoundingBox) -> Self {
        Self { pose, bounds }
    }
}

impl Pose {
    pub fn new(x: f64, y: f64, theta: f64) -> Self {
        Self { x, y, theta }
    }
}

impl Robot {
    pub fn new(pose: Pose, bounds: BoundingBox, cmd: Option<Command>) -> Self {
        Self { pose, bounds, cmd }
    }   

    pub fn exec_cmd(&mut self, dt: f64) {
        /* execute basic 2D diff-drive kinematics based on velocity and timestep */
        if let Some(cmd) = &self.cmd {
            match cmd {
                Command::Move { v } => {
                    self.pose.x += v * self.pose.theta.cos() * dt;
                    self.pose.y += v * self.pose.theta.sin() * dt;
                }
                Command::Turn { omega } => {
                    self.pose.theta += omega * dt;
                }
                Command::Stop => {}
                _ => {}
            }
        }
    }
}
