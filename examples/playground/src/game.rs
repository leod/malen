use coarse_prof::profile;

use malen::{
    al::{self, Sound, SoundSourceNode},
    particles::{Particle, Particles},
    text::Font,
    Color3, Color4, Context, Event, FrameError, InitError, Key, Profile, ProfileParams,
};
use nalgebra::{Point2, Point3, Vector2};
use rand::Rng;

use crate::draw::Draw;
use crate::state::{GameEvent, State};

pub struct Game {
    context: Context,
    profile: Profile,

    shoot_sound: Sound,
    hit_sound: Sound,
    shoot_node: Option<SoundSourceNode>,

    state: State,
    smoke: Particles,
    draw: Draw,

    indirect_light: bool,
    show_profile: bool,
    show_textures: bool,
}

impl Game {
    pub async fn new(context: Context) -> Result<Game, InitError> {
        let font = Font::load(&context, "resources/RobotoMono-Regular.ttf", 40.0).await?;
        let profile = Profile::new(&context, font, ProfileParams::default())?;

        let shoot_sound =
            Sound::load(context.al(), "resources/344276__nsstudios__laser3.wav").await?;
        let hit_sound =
            Sound::load(context.al(), "resources/168984__lavik89__digital-hit.wav").await?;

        let state = State::new();
        let smoke = Particles::new(Vector2::new(512, 512));
        let draw = Draw::new(&context, &state).await?;

        Ok(Game {
            context,
            profile,
            shoot_sound,
            hit_sound,
            shoot_node: None,
            state,
            smoke,
            draw,
            indirect_light: true,
            show_profile: false,
            show_textures: false,
        })
    }

    pub fn frame(&mut self, timestamp_secs: f64) -> Result<(), FrameError> {
        let _frame_guard = self.profile.frame_guard();

        while let Some(event) = self.context.pop_event() {
            self.handle_event(event);
        }

        self.update(timestamp_secs)?;
        self.render()?;
        self.draw()?;

        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        profile!("Game::handle_event");

        use Event::*;
        match event {
            Focused => {
                log::info!("Canvas got focus");
            }
            Unfocused => {
                log::info!("Canvas lost focus");
            }
            KeyPressed(key) => {
                self.state.handle_key_pressed(key);

                match key {
                    Key::P => {
                        log::info!("Profiling:\n{}", coarse_prof::to_string());
                        log::info!(
                            "Frame timer: {:?}",
                            self.profile.draw_timer().borrow().timing_info()
                        );
                        self.show_profile = !self.show_profile;
                        coarse_prof::reset();
                    }
                    Key::U => {
                        self.show_textures = !self.show_textures;
                    }
                    Key::L => {
                        self.indirect_light = !self.indirect_light;
                    }
                    Key::R => {
                        coarse_prof::reset();
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }

    fn handle_game_event(&mut self, game_event: GameEvent) -> Result<(), FrameError> {
        use GameEvent::*;

        match game_event {
            LaserHit { pos, dir } => {
                self.spawn_smoke(pos, dir.y.atan2(dir.x), 0.95 * std::f32::consts::PI, 5);
                al::play_spatial(&self.hit_sound, Point3::new(pos.x, pos.y, 0.0))?;
            }
        }

        Ok(())
    }

    fn update(&mut self, timestamp_secs: f64) -> Result<(), FrameError> {
        profile!("Game::update");

        let (dt_secs, game_events) = self.state.update(
            timestamp_secs,
            self.context.screen(),
            self.context.input_state(),
        );

        for game_event in game_events {
            self.handle_game_event(game_event)?;
        }

        let player_pos = Point3::new(self.state.player.pos.x, self.state.player.pos.y, 0.0);
        self.context.al().set_listener_pos(player_pos);
        match (self.state.player.is_shooting, self.shoot_node.as_ref()) {
            (false, Some(node)) => {
                node.stop().unwrap(); // TODO: wrap web audio
                self.shoot_node = None;
            }
            (true, None) => {
                let node = al::play_spatial(&self.shoot_sound, player_pos)?;
                node.set_loop(true);
                self.shoot_node = Some(node)
            }
            _ => (),
        }

        self.smoke.update(dt_secs);

        Ok(())
    }

    fn render(&mut self) -> Result<(), FrameError> {
        profile!("Game::render");

        self.draw
            .render(self.context.screen(), &self.state, &self.smoke)?;

        Ok(())
    }

    fn draw(&mut self) -> Result<(), FrameError> {
        profile!("Game::draw");

        self.context
            .clear_color_and_depth(Color4::new(1.0, 1.0, 1.0, 1.0), 1.0);
        self.draw.draw(&self.context, self.indirect_light)?;

        if self.show_profile {
            self.profile.draw(self.context.screen())?;
        }
        if self.show_textures {
            self.draw.draw_debug_textures(&mut self.context)?;
        }

        Ok(())
    }

    fn spawn_smoke(&mut self, pos: Point2<f32>, angle: f32, angle_size: f32, n: usize) {
        let mut rng = rand::thread_rng();

        for _ in 0..n {
            let angle = rng.gen_range(angle - angle_size / 2.0, angle + angle_size / 2.0);
            let speed = 1.5 * rng.gen_range(10.0, 100.0);
            let vel = Vector2::new(angle.cos(), angle.sin()) * speed;
            let rot = 0.0; //std::f32::consts::PI * rng.gen_range(-1.0, 1.0);
            let max_age_secs = rng.gen_range(0.7, 1.3);

            let particle = Particle {
                pos,
                angle,
                vel,
                rot,
                depth: 0.15,
                size: Vector2::new(25.0, 25.0),
                color: Color3::new(1.0, 0.8, 0.8).to_linear().to_color4(),
                slowdown: 2.0,
                age_secs: 0.0,
                max_age_secs,
            };

            self.smoke.spawn(particle);
        }
    }
}
