use coarse_prof::profile;
use malen::{
    geom::{shape_shape_overlap, Camera, Circle, Rect, RotatedRect, Screen, Shape},
    InputState, Key,
};
use nalgebra::{Point2, Vector2};
use rand::{prelude::SliceRandom, Rng};

pub const MAP_SIZE: f32 = 2048.0;
pub const ENEMY_RADIUS: f32 = 20.0;
pub const PLAYER_SIZE: f32 = 50.0;

pub struct Wall {
    pub center: Point2<f32>,
    pub size: Vector2<f32>,
}

impl Wall {}

pub struct Enemy {
    pub pos: Point2<f32>,
    pub angle: f32,
    pub rot: f32,
}

pub struct Player {
    pub pos: Point2<f32>,
    pub angle: f32,
}

pub struct Ball {
    pub pos: Point2<f32>,
    pub radius: f32,
}

pub struct State {
    pub walls: Vec<Wall>,
    pub enemies: Vec<Enemy>,
    pub balls: Vec<Ball>,
    pub player: Player,
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

impl Player {
    pub fn rotated_rect(&self) -> RotatedRect {
        RotatedRect {
            center: self.pos,
            size: Vector2::new(PLAYER_SIZE, PLAYER_SIZE),
            angle: self.angle,
        }
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

impl State {
    pub fn new() -> Self {
        let mut state = Self {
            walls: Vec::new(),
            enemies: Vec::new(),
            balls: Vec::new(),
            player: Player {
                pos: Point2::origin(),
                angle: 0.0,
            },
            last_timestamp_secs: None,
        };

        for _ in 0..100 {
            state.add_wall();
        }
        for _ in 0..150 {
            state.add_enemy();
        }
        for _ in 0..50 {
            state.add_ball();
        }

        state
    }

    pub fn shape_overlap(&self, shape: &Shape) -> bool {
        self.walls
            .iter()
            .map(|wall| {
                Shape::Rect(Rect {
                    center: wall.center,
                    size: wall.size,
                })
            })
            .chain(self.balls.iter().map(|ball| {
                Shape::Circle(Circle {
                    center: ball.pos,
                    radius: ball.radius,
                })
            }))
            .chain(self.enemies.iter().map(|enemy| {
                Shape::Circle(Circle {
                    center: enemy.pos,
                    radius: ENEMY_RADIUS,
                })
            }))
            .any(|map_shape| shape_shape_overlap(shape, &map_shape))
    }

    pub fn add_wall(&mut self) {
        let mut rng = rand::thread_rng();
        let center =
            Point2::new(rng.gen(), rng.gen()) * 2.0 * MAP_SIZE - Vector2::new(1.0, 1.0) * MAP_SIZE;

        let size = match rng.gen_range(0, 3) {
            0 => {
                let x = rng.gen_range(50.0, 500.0);
                Vector2::new(x, x)
            }
            1 => Vector2::new(50.0, rng.gen_range(100.0, 1000.0)),
            2 => Vector2::new(rng.gen_range(100.0, 1000.0), 50.0),
            _ => unreachable!(),
        };

        let wall = Wall { center, size };

        if !self.shape_overlap(&wall.shape()) {
            self.walls.push(wall);
        }
    }

    pub fn add_enemy(&mut self) {
        let mut rng = rand::thread_rng();
        let pos =
            Point2::new(rng.gen(), rng.gen()) * 2.0 * MAP_SIZE - Vector2::new(1.0, 1.0) * MAP_SIZE;

        let enemy = Enemy {
            pos,
            angle: rng.gen::<f32>() * std::f32::consts::PI,
            rot: (0.05 + rng.gen::<f32>() * 0.3) * std::f32::consts::PI,
        };

        if !self.shape_overlap(&enemy.shape()) {
            self.enemies.push(enemy);
        }
    }

    pub fn add_ball(&mut self) {
        let mut rng = rand::thread_rng();
        let pos =
            Point2::new(rng.gen(), rng.gen()) * 2.0 * MAP_SIZE - Vector2::new(1.0, 1.0) * MAP_SIZE;
        let radius = *vec![50.0, 100.0].choose(&mut rng).unwrap();

        let ball = Ball { pos, radius };

        if !self.shape_overlap(&ball.shape()) {
            self.balls.push(ball);
        }
    }

    pub fn camera(&self) -> Camera {
        Camera {
            center: self.player.pos,
            zoom: 1.0,
            angle: 0.0,
        }
    }

    pub fn update(&mut self, timestamp_secs: f64, screen: Screen, input_state: &InputState) {
        profile!("update");

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
        if player_dir.norm_squared() > 0.0 {
            let player_dir = player_dir.normalize();
            self.player.pos += dt_secs * 500.0 * player_dir;
        }

        self.player.angle = {
            let mouse_logical_pos = input_state.mouse_logical_pos().cast::<f32>();
            let mouse_world_pos = self
                .camera()
                .inverse_matrix(screen)
                .transform_point(&mouse_logical_pos);
            let offset = mouse_world_pos - self.player.pos;
            offset.y.atan2(offset.x)
        };

        for (i, enemy) in self.enemies.iter_mut().enumerate() {
            let mut delta = enemy.rot * dt_secs;
            if i % 2 == 0 {
                delta *= -1.0;
            }
            enemy.angle += delta;
        }
    }
}
