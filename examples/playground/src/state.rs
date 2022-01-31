use nalgebra::{Point2, Vector2};
use rand::{prelude::SliceRandom, Rng};

use malen::{
    geom::{self, Camera, Circle, Grid, Line, Rect, RotatedRect, Screen, Shape},
    Button, InputState, Key,
};

pub const MAP_SIZE: f32 = 4096.0;
pub const ENEMY_RADIUS: f32 = 20.0;
pub const LAMP_RADIUS: f32 = 12.0;
pub const PLAYER_SIZE: f32 = 35.0;
pub const PLAYER_SHOT_COOLDOWN_SECS: f32 = 0.005;
pub const LASER_LENGTH: f32 = 7.0;
pub const LASER_WIDTH: f32 = 2.0;
pub const LASER_SPEED: f32 = 600.0;

#[derive(Debug, Clone)]
pub struct Wall {
    pub center: Point2<f32>,
    pub size: Vector2<f32>,
    pub lamp_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub pos: Point2<f32>,
    pub vel: Vector2<f32>,
    pub angle: f32,
    pub rot: f32,
    pub bump_power: f32,
    pub bump: f32,
    pub dead: bool,
    pub grid_key: usize,
    pub die_dir: Vector2<f32>,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub pos: Point2<f32>,
    pub vel: Vector2<f32>,
    pub dir: Vector2<f32>,
    pub shot_cooldown_secs: f32,
    pub is_shooting: bool,
}

#[derive(Debug, Clone)]
pub struct Ball {
    pub pos: Point2<f32>,
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct Lamp {
    pub pos: Point2<f32>,
    pub light_angle: f32,
}

#[derive(Debug, Clone)]
pub struct Laser {
    pub pos: Point2<f32>,
    pub vel: Vector2<f32>,
    pub dead: bool,
}

#[derive(Debug, Clone)]
pub enum GameEvent {
    LaserHit {
        entity_type: EntityType,
        pos: Point2<f32>,
        dir: Vector2<f32>,
    },
    EnemyDied {
        pos: Point2<f32>,
        dir: Vector2<f32>,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EntityType {
    Wall,
    Enemy(usize),
    Ball,
    Lamp,
    Laser,
    Player,
}

#[derive(Debug, Clone)]
pub struct State {
    pub walls: Vec<Wall>,
    pub enemies: Vec<Enemy>,
    pub balls: Vec<Ball>,
    pub lamps: Vec<Lamp>,
    pub lasers: Vec<Laser>,
    pub player: Player,
    pub view_offset: Vector2<f32>,
    pub grid: Grid<EntityType>,
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
            radius: ENEMY_RADIUS * (1.0 + self.bump),
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

    pub fn shape(&self) -> Shape {
        Shape::Circle(self.circle())
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
                shot_cooldown_secs: 0.0,
                is_shooting: false,
            },
            view_offset: Vector2::zeros(),
            grid: Grid::new(
                Rect {
                    center: Point2::origin(),
                    size: Vector2::new(MAP_SIZE, MAP_SIZE),
                },
                200.0,
            ),
        };

        for _ in 0..350 {
            state.add_wall();
        }
        for _ in 0..10000 {
            state.add_enemy();
        }
        for _ in 0..50 {
            state.add_ball();
        }
        for _ in 0..300 {
            state.add_lamp();
        }

        log::info!(
            "walls: {}, enemies: {}, balls: {}, lamps: {}",
            state.walls.len(),
            state.enemies.len(),
            state.balls.len(),
            state.lamps.len()
        );

        state
    }

    pub fn camera(&self) -> Camera {
        let center = self.player.pos + self.view_offset;

        Camera {
            center,
            zoom: 3.0,
            angle: 0.0,
        }
    }

    pub fn floor_rect(&self) -> Rect {
        Rect {
            center: Point2::origin(),
            size: Vector2::new(MAP_SIZE, MAP_SIZE),
        }
    }

    pub fn add_wall(&mut self) {
        let mut rng = rand::thread_rng();
        let center = self.floor_rect().sample(&mut rng);

        let one = 30.0;
        let size = match rng.gen_range(0, 3) {
            0 => {
                let x = one * rng.gen_range(1, 4) as f32;
                Vector2::new(x, x)
            }
            1 => Vector2::new(one, one * rng.gen_range(8, 30) as f32),
            2 => Vector2::new(one * rng.gen_range(8, 30) as f32, one),
            _ => unreachable!(),
        };

        let wall = Wall {
            center,
            size,
            lamp_index: None,
        };

        if self.grid.overlap(&wall.shape()).count() == 0 {
            self.grid.insert(wall.shape(), EntityType::Wall);
            self.walls.push(wall);
        }
    }

    pub fn add_enemy(&mut self) {
        let mut rng = rand::thread_rng();
        let pos = self.floor_rect().sample(&mut rng);

        let mut enemy = Enemy {
            pos,
            vel: Vector2::new(0.0, 0.0),
            angle: rng.gen::<f32>() * std::f32::consts::PI,
            rot: (0.05 + rng.gen::<f32>() * 0.15) * std::f32::consts::PI,
            bump_power: 0.0,
            bump: 0.0,
            dead: false,
            grid_key: 0,
            die_dir: Vector2::zeros(),
        };

        if self.grid.overlap(&enemy.shape()).count() == 0 {
            enemy.grid_key = self
                .grid
                .insert(enemy.shape(), EntityType::Enemy(self.enemies.len()));
            self.enemies.push(enemy);
        }
    }

    pub fn add_ball(&mut self) {
        let mut rng = rand::thread_rng();
        let pos = self.floor_rect().sample(&mut rng);
        let radius = *vec![50.0, 100.0].choose(&mut rng).unwrap();

        let ball = Ball { pos, radius };

        if self.grid.overlap(&ball.shape()).count() == 0 {
            self.grid.insert(ball.shape(), EntityType::Ball);
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
                pos: line.0 + 0.5 * (line.1 - line.0) + normal * 5.0,
                light_angle: normal.y.atan2(normal.x),
            };

            self.grid.insert(lamp.shape(), EntityType::Lamp);
            self.lamps.push(lamp);
        }
    }

    pub fn handle_key_pressed(&mut self, _: Key) {}

    pub fn update(
        &mut self,
        dt_secs: f32,
        screen: Screen,
        input_state: &InputState,
    ) -> Vec<GameEvent> {
        let events = self.update_world(dt_secs);
        self.update_player(dt_secs, screen, input_state);
        events
    }

    fn update_player(&mut self, dt_secs: f32, screen: Screen, input_state: &InputState) {
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

        let mut player = self.player.clone();
        for (entry, overlap) in self.grid.overlap(&player.shape()) {
            if let EntityType::Enemy(j) = entry.data {
                if !self.enemies[j].dead {
                    player.pos += 0.01 * overlap.resolution();
                }
            } else {
                player.pos += overlap.resolution();
            }
        }
        self.player = player;

        let mouse_logical_pos = input_state.mouse_logical_pos().cast::<f32>();
        let mouse_world_pos = self
            .camera()
            .inverse_matrix(screen)
            .transform_point(&mouse_logical_pos);

        let player_to_mouse = mouse_world_pos - self.player.pos;
        if player_to_mouse.norm() > 0.1 {
            let target_dir = player_to_mouse.normalize();
            self.player.dir = target_dir - (target_dir - self.player.dir) * (-25.0 * dt_secs).exp();
        }

        self.player.is_shooting = input_state.button(Button::Primary);
        if self.player.is_shooting {
            let mut time_budget = dt_secs;

            while self.player.shot_cooldown_secs < time_budget {
                let mut angle = self.player.dir.y.atan2(self.player.dir.x);
                angle += 0.1 * rand::thread_rng().gen_range(-1.0, 1.0);
                let dir = Vector2::new(angle.cos(), angle.sin());

                let start_pos = self.player.pos + dir * PLAYER_SIZE * 0.5;
                let vel = dir * LASER_SPEED;

                self.lasers.push(Laser {
                    pos: start_pos + (dt_secs - time_budget) * vel,
                    vel,
                    dead: false,
                });

                time_budget -= self.player.shot_cooldown_secs;
                self.player.shot_cooldown_secs = PLAYER_SHOT_COOLDOWN_SECS;
            }

            self.player.shot_cooldown_secs -= time_budget;
        }

        let target_offset = (mouse_logical_pos - screen.logical_rect().center) / 10.0;
        let b = screen.logical_size * 0.3;
        let target_offset = Vector2::new(
            target_offset.x.min(b.x).max(-b.x),
            target_offset.y.min(b.y).max(-b.y),
        );
        self.view_offset =
            target_offset - (target_offset - self.view_offset) * (-3.0 * dt_secs).exp();
    }

    fn update_world(&mut self, dt_secs: f32) -> Vec<GameEvent> {
        let mut events = Vec::new();

        for i in 0..self.enemies.len() {
            if self.enemies[i].dead {
                continue;
            }

            let to_player = self.player.pos - self.enemies[i].pos;
            let target_vel = if (0.1..1000.0).contains(&to_player.norm()) {
                let target_angle = to_player.y.atan2(to_player.x);
                let angle_dist = target_angle - self.enemies[i].angle;
                let angle_dist = angle_dist.sin().atan2(angle_dist.cos());
                self.enemies[i].angle += 0.1 * angle_dist;

                to_player.normalize() * 70.0
            } else {
                let mut delta = self.enemies[i].rot * dt_secs;
                if i % 2 == 0 {
                    delta *= -1.0;
                }
                self.enemies[i].angle += delta;
                Vector2::zeros()
            };

            let delta = dt_secs * self.enemies[i].vel;
            self.enemies[i].vel =
                target_vel - (target_vel - self.enemies[i].vel) * (-10.0 * dt_secs).exp();
            self.enemies[i].pos += delta;

            self.grid.remove(self.enemies[i].grid_key);

            for (entry, overlap) in self.grid.overlap(&self.enemies[i].shape()) {
                let delta = match entry.data {
                    EntityType::Enemy(j) if !self.enemies[j].dead => 0.2 * overlap.resolution(),
                    _ => overlap.resolution(),
                };
                self.enemies[i].pos += delta;
            }
            if let Some(overlap) = geom::rotated_rect_circle_overlap(
                self.player.rotated_rect(),
                self.enemies[i].circle(),
            ) {
                self.enemies[i].pos -= 0.2 * overlap.resolution();
            }

            self.enemies[i].grid_key = self
                .grid
                .insert(self.enemies[i].shape(), EntityType::Enemy(i));

            self.enemies[i].bump += self.enemies[i].bump_power * dt_secs;
            self.enemies[i].bump_power *= (-10.0 * dt_secs).exp();
            self.enemies[i].bump *= (-10.0 * dt_secs).exp();

            if self.enemies[i].bump > 0.9 {
                self.enemies[i].dead = true;
                self.grid.remove(self.enemies[i].grid_key);
                events.push(GameEvent::EnemyDied {
                    pos: self.enemies[i].pos,
                    dir: self.enemies[i].die_dir,
                });
            }
        }

        for i in 0..self.lasers.len() {
            let vel = self.lasers[i].vel;
            self.lasers[i].pos += vel * dt_secs;

            for (entry, overlap) in self.grid.overlap(&self.lasers[i].shape()) {
                if let EntityType::Enemy(j) = entry.data {
                    if self.enemies[j].dead {
                        continue;
                    }
                    self.enemies[j].bump_power += 0.6;
                    self.enemies[j].die_dir =
                        0.5 * self.lasers[i].vel.normalize() + 0.5 * self.enemies[j].die_dir;
                    self.enemies[j].die_dir.normalize_mut();
                }

                events.push(GameEvent::LaserHit {
                    entity_type: entry.data,
                    pos: self.lasers[i].line().1 + overlap.resolution(),
                    dir: overlap.resolution().normalize(),
                });
                self.lasers[i].dead = true;
            }

            let out_of_bounds = !self.floor_rect().contains_point(self.lasers[i].line().0)
                && !self.floor_rect().contains_point(self.lasers[i].line().0);
            if out_of_bounds {
                self.lasers[i].dead = true;
            }
        }

        self.lasers.retain(|laser| !laser.dead);

        events
    }
}
