use coarse_prof::profile;

use malen::{
    al::{self, ReverbNode, ReverbParams, Sound, SpatialPlayNode, SpatialPlayParams},
    geom::{self, Circle, Shape},
    particles::{Particle, Particles},
    text::{Font, Text},
    Color3, Color4, Context, Event, FrameError, InitError, Key, Profile, ProfileParams,
};
use nalgebra::{Point2, Point3, Vector2, Vector3};
use rand::Rng;

use crate::state::{GameEvent, State};
use crate::{draw::Draw, state::EntityType};

pub struct Game {
    context: Context,
    profile: Profile,

    shoot_sound: Sound,
    hit1_sound: Sound,
    hit2_sound: Sound,
    explosion_sound: Sound,
    reverb: ReverbNode,

    state: State,
    smoke: Particles,
    draw: Draw,

    last_timestamp_secs: Option<f64>,
    hit_sound_cooldown_secs: f32,
    shoot_node: Option<SpatialPlayNode>,

    indirect_light: bool,
    show_profile: bool,
    show_textures: bool,

    update_budget_secs: f32,
}

impl Game {
    pub async fn new(context: Context) -> Result<Game, InitError> {
        let font = Font::load(&context, "resources/RobotoMono-Regular.ttf", 40.0).await?;
        let profile = Profile::new(&context, font, ProfileParams::default())?;

        let shoot_sound = Sound::load(
            context.al(),
            "resources/440143__dpren__scifi-gun-laser-automatic-fast_cut.wav",
        )
        .await?;
        let hit1_sound =
            Sound::load(context.al(), "resources/344276__nsstudios__laser3.wav").await?;
        let hit2_sound = Sound::load(
            context.al(),
            "resources/612877__sound-designer-from-turkey__laser-1.wav",
        )
        .await?;
        let explosion_sound = Sound::load(
            context.al(),
            "resources/183869__m-red__darkdetonation02.wav",
        )
        .await?;
        let impulse = Sound::load(context.al(), "resources/impulse4.wav").await?;
        let reverb = al::reverb(
            &impulse,
            context.al().destination(),
            &ReverbParams::default(),
        )?;

        let state = State::new();
        let smoke = Particles::new(Vector2::new(512, 512));
        let draw = Draw::new(&context, &state).await?;

        Ok(Game {
            context,
            profile,
            shoot_sound,
            hit1_sound,
            hit2_sound,
            explosion_sound,
            reverb,
            state,
            smoke,
            draw,
            last_timestamp_secs: None,
            hit_sound_cooldown_secs: 0.0,
            shoot_node: None,
            indirect_light: true,
            show_profile: false,
            show_textures: false,
            update_budget_secs: 0.0,
        })
    }

    pub fn frame(&mut self, timestamp_secs: f64) -> Result<(), FrameError> {
        let dt_secs = self
            .last_timestamp_secs
            .map_or(0.0, |last_timestamp_secs| {
                (timestamp_secs - last_timestamp_secs) as f32
            })
            .max(0.0);
        self.last_timestamp_secs = Some(timestamp_secs);

        let _frame_guard = self.profile.frame_guard(dt_secs);

        while let Some(event) = self.context.pop_event() {
            self.handle_event(event);
        }

        let update_secs = 1.0 / 60.0;
        self.update_budget_secs = (self.update_budget_secs + dt_secs).min(2.0 * update_secs);

        if self.update_budget_secs >= update_secs {
            let update_speed = 1.5;
            // TODO: We should consider a fixed dt_secs here, since we have some effects
            //       involving exponentials. However, then we might also need to interpolate
            //       states, which I'm too lazy to implement now.
            self.update(update_speed * update_secs)?;
            self.update_budget_secs -= update_secs;
        }

        self.render()?;
        self.draw()?;

        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        profile!("handle_event");

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
                        log::info!("Profiling:\n{}", coarse_prof::to_string());
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
            LaserHit {
                entity_type,
                pos,
                dir,
            } => {
                self.spawn_smoke(pos, dir.y.atan2(dir.x), 0.95 * std::f32::consts::PI, 5);
                if self.hit_sound_cooldown_secs == 0.0 {
                    let hit_sound = match entity_type {
                        EntityType::Ball | EntityType::Enemy(_) => &self.hit1_sound,
                        _ => &self.hit2_sound,
                    };
                    let gain = match entity_type {
                        EntityType::Ball | EntityType::Enemy(_) => 0.4,
                        _ => 1.0,
                    };
                    al::play_spatial(
                        hit_sound,
                        &SpatialPlayParams {
                            cone_inner_angle: 180.0,
                            cone_outer_angle: 90.0,
                            orientation: Vector3::new(dir.x, dir.y, 0.0),
                            pos: Point3::new(pos.x, pos.y, 0.0),
                            gain,
                            ..SpatialPlayParams::default()
                        },
                        self.reverb.input(),
                    )?;
                    self.hit_sound_cooldown_secs = 0.05;
                }
            }
            EnemyDied { pos, dir } => {
                self.spawn_smoke_explosion(pos, dir, 200);
                al::play_spatial(
                    &self.explosion_sound,
                    &SpatialPlayParams {
                        pos: Point3::new(pos.x, pos.y, 0.0),
                        gain: 1.0,
                        ..SpatialPlayParams::default()
                    },
                    self.reverb.input(),
                )?;
            }
        }

        Ok(())
    }

    fn update(&mut self, dt_secs: f32) -> Result<(), FrameError> {
        profile!("update");

        let game_events =
            self.state
                .update(dt_secs, self.context.screen(), self.context.input_state());

        {
            profile!("handle_events");
            for game_event in game_events {
                self.handle_game_event(game_event)?;
            }

            /*if self.state.player.is_shooting {
                self.spawn_smoke(
                    self.state.player.pos + self.state.player.dir * 22.0,
                    self.state.player.dir.y.atan2(self.state.player.dir.x) + std::f32::consts::PI,
                    std::f32::consts::PI,
                    1,
                );
            }*/
        }

        self.update_audio(dt_secs)?;
        {
            profile!("particles");
            {
                profile!("overlap");
                let player_circle = self.state.player.rotated_rect().bounding_circle();

                for particle in self.smoke.iter_mut() {
                    let circle = Circle {
                        center: particle.pos,
                        radius: 1.0,
                    };
                    for (entry, overlap) in self.state.grid.overlap(&Shape::Circle(circle)) {
                        let speed = match entry.data {
                            EntityType::Wall => 15.0,
                            _ => 30.0,
                        };
                        particle.vel += overlap.resolution().normalize() * speed;
                        particle.slowdown *= 1.25;
                    }

                    if geom::circle_circle_overlap(circle, player_circle).is_some() {
                        if let Some(overlap) = geom::rotated_rect_circle_overlap(
                            self.state.player.rotated_rect(),
                            circle,
                        ) {
                            particle.vel -= overlap.resolution().normalize() * 50.0;
                            particle.vel += self.state.player.vel * 0.1;
                            particle.slowdown *= 1.25;
                        }
                    }
                }
            }
            {
                profile!("update");
                self.smoke.update(dt_secs);
            }
        }

        Ok(())
    }

    fn update_audio(&mut self, dt_secs: f32) -> Result<(), FrameError> {
        profile!("update_audio");

        let player_pos = Point3::new(self.state.player.pos.x, self.state.player.pos.y, 0.0);
        self.context.al().set_listener_pos(player_pos);

        match (self.state.player.is_shooting, self.shoot_node.as_ref()) {
            (false, Some(node)) => {
                node.set_loop(false);
            }
            (true, node) => {
                if node.map_or(true, |node| !node.source.loop_()) {
                    let node = al::play_spatial(
                        &self.shoot_sound,
                        &SpatialPlayParams {
                            pos: player_pos,
                            gain: 0.4,
                            ..SpatialPlayParams::default()
                        },
                        self.reverb.input(),
                    )?;
                    node.source.set_loop_start(0.05);
                    node.source.set_loop_end(0.11);
                    node.set_loop(true);
                    self.shoot_node = Some(node)
                }
            }
            _ => (),
        }

        if let Some(node) = self.shoot_node.as_ref() {
            node.set_pos(player_pos);
        }

        self.hit_sound_cooldown_secs = (self.hit_sound_cooldown_secs - dt_secs).max(0.0);

        Ok(())
    }

    fn render(&mut self) -> Result<(), FrameError> {
        profile!("render");

        let render_info = self
            .draw
            .render(self.context.screen(), &self.state, &self.smoke)?;

        /*let dists = self
            .draw
            .light_pipeline
            .shadow_map_framebuffer()
            .read_pixel_row_f16(0, 0);
        let avg_dist: f32 = dists.iter().copied().map(f32::from).sum::<f32>() / dists.len() as f32;
        let max_dist: f32 = dists
            .iter()
            .copied()
            .map(f32::from)
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap();
        let closed_perc: f32 =
            dists.iter().filter(|&&d| f32::from(d) < 0.7).count() as f32 / dists.len() as f32;
        let reverb_params = ReverbParams {
            pre_delay_secs: 0.3 * avg_dist.powf(2.0),
            num_taps: ((closed_perc - 0.5).max(0.0) * 20.0) as usize + 1,
            convolver_gain: 0.1 + 0.2 * avg_dist.powf(2.0),
            ..ReverbParams::default()
        };
        self.reverb.linear_ramp_to_params(&reverb_params, 0.05)?;
        self.draw.font.write(
            Text {
                pos: Point2::new(10.0, 10.0),
                size: 20.0,
                z: 0.0,
                color: Color4::new(1.0, 1.0, 1.0, 1.0),
                text: &format!(
                    "avg_dist: {:.4}\navg_dist_sq: {:.4}\nmax_dist: {:.4}\nclosed_perc: {:.4}\nreverb_params: {:?}",
                    avg_dist,
                    avg_dist.powf(2.0),
                    max_dist,
                    closed_perc,
                    reverb_params,
                ),
            },
            &mut self.draw.text_batch,
        )?;*/

        if self.show_profile {
            self.draw.font.write(
                Text {
                    pos: Point2::new(self.context.screen().logical_size.x - 300.0, 10.0),
                    size: 12.0,
                    z: 0.0,
                    color: Color4::new(1.0, 1.0, 1.0, 1.0),
                    text: &format!("{:#?}\n{:#?}", render_info, self.state.grid.info()),
                },
                &mut self.draw.text_batch,
            )?;
        }

        Ok(())
    }

    fn draw(&mut self) -> Result<(), FrameError> {
        profile!("draw");

        self.context
            .clear_color_and_depth(Color4::new(1.0, 1.0, 1.0, 1.0), 1.0);
        self.draw.draw(&self.context, self.indirect_light)?;

        if self.show_profile {
            profile!("profile");
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
            let max_age_secs = rng.gen_range(0.7, 1.3);

            let particle = Particle {
                pos,
                angle,
                vel,
                depth: 0.15,
                size: Vector2::new(25.0, 25.0),
                color: Color3::new(1.0, 1.0, 1.0).to_linear().to_color4(),
                slowdown: 2.0,
                age_secs: 0.0,
                max_age_secs,
            };

            self.smoke.spawn(particle);
        }
    }

    fn spawn_smoke_explosion(&mut self, pos: Point2<f32>, _dir: Vector2<f32>, n: usize) {
        let mut rng = rand::thread_rng();

        for _ in 0..n {
            let speed = 0.4 * rng.gen_range(5.0, 150.0);
            let angle = rng.gen_range(0.0, std::f32::consts::PI * 2.0);
            let vel = Vector2::new(angle.cos(), angle.sin()) * speed;
            let max_age_secs = rng.gen_range(3.0, 8.0);

            let particle = Particle {
                pos,
                angle: 0.0,
                vel,
                depth: 0.15,
                size: Vector2::new(30.0, 30.0),
                color: Color3::new(1.0, 0.8, 0.8).to_linear().to_color4(),
                slowdown: 1.0,
                age_secs: 0.0,
                max_age_secs,
            };

            self.smoke.spawn(particle);
        }

        for _ in 0..n {
            let speed = 1.5 * rng.gen_range(400.0, 500.0);
            let angle = std::f32::consts::PI * rng.gen_range(-1.0, 1.0);
            let vel = Vector2::new(angle.cos(), angle.sin()) * speed;
            let max_age_secs = 2.0 * rng.gen_range(0.6, 0.8);
            let size = 12.5 * rng.gen_range(0.5, 4.5);

            let particle = Particle {
                pos,
                angle: 0.0,
                vel,
                depth: 0.25,
                size: Vector2::new(size, size),
                color: Color3::new(0.9, 0.4, 0.4).to_linear().to_color4(),
                slowdown: 10.0,
                age_secs: 0.0,
                max_age_secs,
            };

            self.smoke.spawn(particle);
        }
    }
}
