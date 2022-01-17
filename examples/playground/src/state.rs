use nalgebra::{Point2, Vector2};
use rand::{prelude::SliceRandom, Rng};

use malen::{
    geom::{shape_shape_overlap, Camera, Circle, Line, Overlap, Rect, RotatedRect, Screen, Shape},
    InputState, Key,
};

pub const MAP_SIZE: f32 = 4096.0;
pub const ENEMY_RADIUS: f32 = 20.0;
pub const LAMP_RADIUS: f32 = 15.0;
pub const PLAYER_SIZE: f32 = 35.0;
pub const LASER_LENGTH: f32 = 25.0;
pub const LASER_WIDTH: f32 = 3.0;
pub const LASER_SPEED: f32 = 350.0;

pub struct Wall {
    pub center: Point2<f32>,
    pub size: Vector2<f32>,
    pub lamp_index: Option<usize>,
    pub use_texture: bool,
}

pub struct Enemy {
    pub pos: Point2<f32>,
    pub angle: f32,
    pub rot: f32,
}

pub struct Player {
    pub pos: Point2<f32>,
    pub vel: Vector2<f32>,
    pub dir: Vector2<f32>,
}

pub struct Ball {
    pub pos: Point2<f32>,
    pub radius: f32,
}

pub struct Lamp {
    pub pos: Point2<f32>,
    pub light_angle: f32,
}

pub struct Laser {
    pub pos: Point2<f32>,
    pub vel: Vector2<f32>,
    pub dead: bool,
}

pub struct State {
    pub walls: Vec<Wall>,
    pub enemies: Vec<Enemy>,
    pub balls: Vec<Ball>,
    pub lamps: Vec<Lamp>,
    pub lasers: Vec<Laser>,
    pub player: Player,
    pub view_offset: Vector2<f32>,
    pub last_timestamp_secs: Option<f64>,
}

impl Wall {
    pub fn rect(&self) -> Rect {
        Rect {
            center: self.center,
            size: self.size,
        }
    }

    pub fn shape(&self) -> Shape {
        Shape::Rect(self.rect())
    }
}

impl Enemy {
    pub fn circle(&self) -> Circle {
        Circle {
            center: self.pos,
            radius: ENEMY_RADIUS,
        }
    }

    pub fn shape(&self) -> Shape {
        Shape::Circle(self.circle())
    }
}

impl Ball {
    pub fn circle(&self) -> Circle {
        Circle {
            center: self.pos,
            radius: self.radius,
        }
    }

    pub fn shape(&self) -> Shape {
        Shape::Circle(Circle {
            center: self.pos,
            radius: self.radius,
        })
    }
}

impl Lamp {
    pub fn circle(&self) -> Circle {
        Circle {
            center: self.pos,
            radius: LAMP_RADIUS,
        }
    }
}

impl Laser {
    pub fn line(&self) -> Line {
        Line(self.pos, self.pos + self.vel.normalize() * LASER_LENGTH)
    }

    pub fn rotated_rect(&self) -> RotatedRect {
        Rect {
            center: self.pos,
            size: Vector2::new(LASER_LENGTH, LASER_WIDTH),
        }
        .translate(self.vel.normalize() * LASER_LENGTH / 2.0)
        .rotate(self.vel.y.atan2(self.vel.x))
    }

    pub fn shape(&self) -> Shape {
        Shape::RotatedRect(self.rotated_rect())
    }
}

impl Player {
    pub fn rotated_rect(&self) -> RotatedRect {
        RotatedRect {
            center: self.pos,
            size: Vector2::new(PLAYER_SIZE, PLAYER_SIZE),
            angle: self.dir.y.atan2(self.dir.x),
        }
    }

    pub fn shape(&self) -> Shape {
        Shape::RotatedRect(self.rotated_rect())
    }
}

impl State {
    pub fn new() -> Self {
        let mut state = Self {
            walls: Vec::new(),
            enemies: Vec::new(),
            balls: Vec::new(),
            lamps: Vec::new(),
            lasers: Vec::new(),
            player: Player {
                pos: Point2::origin(),
                vel: Vector2::zeros(),
                dir: Vector2::zeros(),
            },
            view_offset: Vector2::zeros(),
            last_timestamp_secs: None,
        };

        for _ in 0..200 {
            state.add_wall();
        }
        for _ in 0..80 {
            state.add_enemy();
        }
        for _ in 0..50 {
            state.add_ball();
        }
        for _ in 0..40 {
            state.add_lamp();
        }

        state
    }

    pub fn camera(&self) -> Camera {
        Camera {
            center: self.player.pos + self.view_offset,
            zoom: 2.5,
            angle: 0.0,
        }
    }

    pub fn floor_rect(&self) -> Rect {
        Rect {
            center: Point2::origin(),
            size: Vector2::new(MAP_SIZE, MAP_SIZE),
        }
    }

    pub fn shape_overlap(&self, shape: &Shape) -> Option<Overlap> {
        self.walls
            .iter()
            .map(Wall::shape)
            .chain(self.balls.iter().map(Ball::shape))
            .chain(self.enemies.iter().map(Enemy::shape))
            .filter_map(|map_shape| shape_shape_overlap(shape, &map_shape))
            .max_by(|o1, o2| {
                o1.resolution()
                    .norm_squared()
                    .partial_cmp(&o2.resolution().norm_squared())
                    .unwrap()
            })
    }

    pub fn add_wall(&mut self) {
        let mut rng = rand::thread_rng();
        let center = self.floor_rect().sample(&mut rng);

        let size = match rng.gen_range(0, 3) {
            0 => {
                let x = 50.0 * rng.gen_range(1, 10) as f32;
                Vector2::new(x, x)
            }
            1 => Vector2::new(50.0, 50.0 * rng.gen_range(2, 20) as f32),
            2 => Vector2::new(50.0 * rng.gen_range(2, 20) as f32, 50.0),
            _ => unreachable!(),
        };

        let wall = Wall {
            center,
            size,
            lamp_index: None,
            //use_texture: rng.gen(),
            use_texture: true,
        };

        if self.shape_overlap(&wall.shape()).is_none() {
            self.walls.push(wall);
        }
    }

    pub fn add_enemy(&mut self) {
        let mut rng = rand::thread_rng();
        let pos = self.floor_rect().sample(&mut rng);

        let enemy = Enemy {
            pos,
            angle: rng.gen::<f32>() * std::f32::consts::PI,
            rot: (0.05 + rng.gen::<f32>() * 0.15) * std::f32::consts::PI,
        };

        if self.shape_overlap(&enemy.shape()).is_none() {
            self.enemies.push(enemy);
        }
    }

    pub fn add_ball(&mut self) {
        let mut rng = rand::thread_rng();
        let pos = self.floor_rect().sample(&mut rng);
        let radius = *vec![50.0, 100.0].choose(&mut rng).unwrap();

        let ball = Ball { pos, radius };

        if self.shape_overlap(&ball.shape()).is_none() {
            self.balls.push(ball);
        }
    }

    pub fn add_lamp(&mut self) {
        let mut rng = rand::thread_rng();
        let mut empty_walls: Vec<_> = self
            .walls
            .iter_mut()
            .filter(|wall| wall.lamp_index.is_none())
            .collect();

        if let Some(empty_wall) = empty_walls.choose_mut(&mut rng) {
            (*empty_wall).lamp_index = Some(self.lamps.len());

            let lines = empty_wall.rect().edges();
            let line = lines.choose(&mut rng).unwrap();
            let normal = Vector2::new(line.1.y - line.0.y, line.0.x - line.1.x).normalize();
            let lamp = Lamp {
                pos: line.0 + 0.5 * (line.1 - line.0) - normal * 25.0,
                light_angle: normal.y.atan2(normal.x),
            };
            self.lamps.push(lamp);
        }
    }

    pub fn handle_key_pressed(&mut self, key: Key) {
        match key {
            Key::Space => self.lasers.push(Laser {
                pos: self.player.pos + self.player.dir * PLAYER_SIZE * 0.5,
                vel: self.player.dir * LASER_SPEED,
                dead: false,
            }),
            _ => (),
        }
    }

    pub fn update(&mut self, timestamp_secs: f64, screen: Screen, input_state: &InputState) {
        let dt_secs = self
            .last_timestamp_secs
            .map_or(0.0, |last_timestamp_secs| {
                timestamp_secs - last_timestamp_secs
            })
            .max(0.0) as f32;
        self.last_timestamp_secs = Some(timestamp_secs);

        let mut player_dir = Vector2::zeros();
        if input_state.key(Key::W) {
            player_dir.y -= 1.0;
        }
        if input_state.key(Key::S) {
            player_dir.y += 1.0;
        }
        if input_state.key(Key::A) {
            player_dir.x -= 1.0;
        }
        if input_state.key(Key::D) {
            player_dir.x += 1.0;
        }
        let target_vel = if player_dir.norm_squared() > 0.0 {
            let player_dir = player_dir.normalize();
            player_dir * 275.0
        } else {
            Vector2::zeros()
        };

        self.player.vel = target_vel - (target_vel - self.player.vel) * (-25.0 * dt_secs).exp();
        self.player.pos += dt_secs * self.player.vel;

        if let Some(overlap) = self.shape_overlap(&self.player.shape()) {
            self.player.pos -= overlap.resolution();
        }

        let mouse_logical_pos = input_state.mouse_logical_pos().cast::<f32>();
        let mouse_world_pos = self
            .camera()
            .inverse_matrix(screen)
            .transform_point(&mouse_logical_pos);

        let target_dir = (mouse_world_pos - self.player.pos).normalize();
        self.player.dir = target_dir - (target_dir - self.player.dir) * (-25.0 * dt_secs).exp();

        let target_offset = (mouse_logical_pos - screen.logical_rect().center) / 10.0;
        let b = screen.logical_size * 0.3;
        let target_offset = Vector2::new(
            target_offset.x.min(b.x).max(-b.x),
            target_offset.y.min(b.y).max(-b.y),
        );
        self.view_offset =
            target_offset - (target_offset - self.view_offset) * (-3.0 * dt_secs).exp();

        for (i, enemy) in self.enemies.iter_mut().enumerate() {
            let mut delta = enemy.rot * dt_secs;
            if i % 2 == 0 {
                delta *= -1.0;
            }
            enemy.angle += delta;
        }

        for i in 0..self.lasers.len() {
            let vel = self.lasers[i].vel;
            self.lasers[i].pos += vel * dt_secs;

            let overlap = self.shape_overlap(&self.lasers[i].shape()).is_some();
            let out_of_bounds = !self.floor_rect().contains_point(self.lasers[i].line().0)
                && !self.floor_rect().contains_point(self.lasers[i].line().0);

            if overlap || out_of_bounds {
                self.lasers[i].dead = true;
            }
        }

        self.lasers.retain(|laser| !laser.dead);
    }
}
