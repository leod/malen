use coarse_prof::profile;

use malen::text::Font;
use malen::{Color4, Context, Event, FrameError, InitError, Key, Profile, ProfileParams};

use crate::draw::Draw;
use crate::state::State;

pub struct Game {
    context: Context,
    state: State,
    draw: Draw,
    profile: Profile,

    indirect_light: bool,
    show_profile: bool,
    show_textures: bool,
}

impl Game {
    pub async fn new(context: Context) -> Result<Game, InitError> {
        let state = State::new();
        let draw = Draw::new(&context, &state).await?;
        let font = Font::load(&context, "resources/RobotoMono-Regular.ttf", 40.0).await?;
        let profile = Profile::new(&context, font, ProfileParams::default())?;

        Ok(Game {
            context,
            state,
            draw,
            profile,
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

        self.update(timestamp_secs);
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

    fn update(&mut self, timestamp_secs: f64) {
        profile!("Game::update");

        self.state.update(
            timestamp_secs,
            self.context.screen(),
            self.context.input_state(),
        );
    }

    fn render(&mut self) -> Result<(), FrameError> {
        profile!("Game::render");

        self.draw.render(self.context.screen(), &self.state)?;

        Ok(())
    }

    fn draw(&mut self) -> Result<(), FrameError> {
        profile!("Game::draw");

        self.context
            .clear_color_and_depth(Color4::new(1.0, 1.0, 1.0, 1.0), 1.0);
        self.draw.draw(self.indirect_light)?;

        if self.show_profile {
            self.profile.draw(self.context.screen())?;
        }
        if self.show_textures {
            self.draw.draw_debug_textures(&mut self.context)?;
        }

        Ok(())
    }
}
