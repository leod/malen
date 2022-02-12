use coarse_prof::profile;
use nalgebra::{Matrix3, Point2, Point3, Vector2};

use malen::{
    data::{
        ColorCircle, ColorRect, ColorRotatedRect, ColorTriangleBatch, ColorVertex, InstanceBatch,
        Mesh, RotatedSprite, SpriteBatch, TriangleTag, ColorLineBatch,
    },
    geom::{self, Circle, Rect, RotatedRect, Screen},
    gl::{Blend, DepthTest, DrawParams, Texture, TextureParams, Uniform},
    light::{
        GlobalLightProps, Light, LightPipeline, LightPipelineParams, ObjectLightProps,
        OccluderBatch, OccluderCircle, OccluderRect, OccluderRotatedRect,
    },
    particles::Particles,
    pass::{ColorInstance, ViewMatrices},
    text::{Font, TextBatch},
    Color3, Color4, Context, FrameError, InitError,
};

use crate::state::{Ball, Enemy, Lamp, Laser, Player, State, Wall, ENEMY_RADIUS};

const MAX_LIGHT_RADIUS: f32 = 600.0;

#[derive(Debug, Clone)]
pub struct RenderInfo {
    pub translucent_color_verts: usize,
    pub translucent_color_elems: usize,
    pub reflector_color_verts: usize,
    pub reflector_color_elems: usize,
    pub source_color_verts: usize,
    pub source_color_elems: usize,
    pub occluder_verts: usize,
    pub occluder_elems: usize,
    pub lights: usize,
    pub particles: usize,
}

pub struct Draw {
    pub font: Font,
    smoke_texture: Texture,
    smoke_normal_texture: Texture,
    enemy_texture: Texture,
    enemy_normal_texture: Texture,

    pub light_pipeline: LightPipeline,

    translucent_light_props: Uniform<ObjectLightProps>,
    reflector_light_props: Uniform<ObjectLightProps>,
    smoke_light_props: Uniform<ObjectLightProps>,
    camera_matrices: Uniform<ViewMatrices>,
    screen_matrices: Uniform<ViewMatrices>,

    circle_instances: InstanceBatch<ColorVertex, ColorInstance>,
    translucent_color_batch: ColorTriangleBatch,
    reflector_color_batch: ColorTriangleBatch,
    reflector_color_batch2: ColorTriangleBatch,
    reflector_sprite_batch: SpriteBatch,
    source_color_batch: ColorTriangleBatch,
    occluder_batch: OccluderBatch,
    smoke_batch: SpriteBatch,
    outline_batch: ColorLineBatch,
    lights: Vec<Light>,
    pub text_batch: TextBatch,
}

impl Draw {
    pub async fn new(context: &Context, _: &State) -> Result<Draw, InitError> {
        let font = Font::load(context, "resources/RobotoMono-Regular.ttf", 40.0).await?;
        let smoke_texture = Texture::load(
            context.gl(),
            "resources/smoke1.png",
            TextureParams::linear_mipmapped_rgbau8(),
        )
        .await?;
        let smoke_normal_texture = Texture::load(
            context.gl(),
            "resources/smoke1_Nrm.png",
            TextureParams::linear_mipmapped_rgbau8(),
        )
        .await?;
        let enemy_texture = Texture::load(
            context.gl(),
            "resources/enemy.png",
            TextureParams::linear_mipmapped_rgbau8(),
        )
        .await?;
        let enemy_normal_texture = Texture::load(
            context.gl(),
            "resources/enemy_Nrm.png",
            TextureParams::linear_mipmapped_rgbau8(),
        )
        .await?;

        let light_pipeline = LightPipeline::new(context, LightPipelineParams::default())?;

        let translucent_light_props = Uniform::new(
            context.gl(),
            ObjectLightProps {
                occlusion: 0.0,
                reflectance: 0.0,
            },
        )?;
        let reflector_light_props = Uniform::new(
            context.gl(),
            ObjectLightProps {
                occlusion: 1.0,
                reflectance: 200.0,
            },
        )?;
        let smoke_light_props = Uniform::new(
            context.gl(),
            ObjectLightProps {
                occlusion: 0.7,
                reflectance: 20.0,
            },
        )?;
        let camera_matrices = Uniform::new(context.gl(), ViewMatrices::default())?;
        let screen_matrices = Uniform::new(context.gl(), ViewMatrices::default())?;

        let circle_mesh = Mesh::from_geometry::<TriangleTag, _>(
            context.gl(),
            ColorCircle {
                circle: Circle {
                    center: Point2::origin(),
                    radius: 20.0,
                },
                depth: 0.0,
                angle: 0.0,
                num_segments: 64,
                color: Color4::new(1.0, 1.0, 1.0, 1.0),
            },
        )?;

        let circle_instances = InstanceBatch::from_mesh(circle_mesh)?;
        let translucent_color_batch = ColorTriangleBatch::new(context.gl())?;
        let reflector_color_batch = ColorTriangleBatch::new(context.gl())?;
        let reflector_color_batch2 = ColorTriangleBatch::new(context.gl())?;
        let reflector_sprite_batch = SpriteBatch::new(context.gl())?;
        let source_color_batch = ColorTriangleBatch::new(context.gl())?;
        let occluder_batch = light_pipeline.new_occluder_batch()?;
        let smoke_batch = SpriteBatch::new(context.gl())?;
        let outline_batch = ColorLineBatch::new(context.gl())?;
        let lights = Vec::new();
        let text_batch = TextBatch::new(context.gl())?;

        Ok(Draw {
            font,
            smoke_texture,
            smoke_normal_texture,
            enemy_texture,
            enemy_normal_texture,
            light_pipeline,
            translucent_light_props,
            reflector_light_props,
            smoke_light_props,
            camera_matrices,
            screen_matrices,
            circle_instances,
            translucent_color_batch,
            reflector_color_batch,
            reflector_color_batch2,
            reflector_sprite_batch,
            source_color_batch,
            occluder_batch,
            smoke_batch,
            outline_batch,
            lights,
            text_batch,
        })
    }

    pub fn render(
        &mut self,
        screen: Screen,
        state: &State,
        smoke: &Particles,
    ) -> Result<RenderInfo, FrameError> {
        profile!("render");

        self.camera_matrices.set(ViewMatrices {
            view: state.camera().matrix(screen),
            projection: screen.project_logical_to_ndc(),
        });
        self.screen_matrices.set(ViewMatrices {
            view: Matrix3::identity(),
            projection: screen.project_logical_to_ndc(),
        });

        let visible_rect = state.camera().visible_world_rect(screen).scale(1.0);
        let light_rect = visible_rect
            .enlarge(Vector2::new(1.0, 1.0) * 2.0 * MAX_LIGHT_RADIUS / state.camera().zoom);

        self.circle_instances.clear();
        self.translucent_color_batch.clear();
        self.reflector_color_batch.clear();
        self.reflector_color_batch2.clear();
        self.reflector_sprite_batch.clear();
        self.source_color_batch.clear();
        self.text_batch.clear();
        self.occluder_batch.clear();
        self.smoke_batch.clear();
        self.outline_batch.clear();
        self.lights.clear();

        self.render_player(&state.player);
        self.render_floor(state);
        for wall in &state.walls {
            self.render_wall(wall, visible_rect, light_rect);
        }
        for lamp in &state.lamps {
            self.render_lamp(lamp, visible_rect);
        }
        for enemy in &state.enemies {
            self.render_enemy(enemy, visible_rect, light_rect);
        }
        for ball in &state.balls {
            self.render_ball(ball, visible_rect, light_rect);
        }
        for laser in &state.lasers {
            self.render_laser(laser);
        }

        self.smoke_batch.push(smoke);

        let render_info = RenderInfo {
            translucent_color_verts: self.translucent_color_batch.num_vertices(),
            translucent_color_elems: self.translucent_color_batch.num_elements(),
            reflector_color_verts: self.reflector_color_batch.num_vertices(),
            reflector_color_elems: self.reflector_color_batch.num_elements(),
            source_color_verts: self.source_color_batch.num_vertices(),
            source_color_elems: self.source_color_batch.num_elements(),
            occluder_verts: self.occluder_batch.num_vertices(),
            occluder_elems: self.occluder_batch.num_elements(),
            lights: self.lights.len(),
            particles: smoke.len(),
        };

        Ok(render_info)
    }

    fn render_player(&mut self, player: &Player) {
        let color = Color3::from_u8(255, 209, 102);
        self.occluder_batch.push(OccluderRotatedRect {
            rect: player.rotated_rect(),
            height: 1000.0,
            ignore_light_index1: Some(self.lights.len() as u32),
            ignore_light_index2: None,
        });
        self.reflector_color_batch.push(ColorRotatedRect {
            rect: player.rotated_rect(),
            depth: 0.4,
            color: color.to_color4(),
        });
        self.outline_batch.push(ColorRotatedRect {
            rect: player.rotated_rect(),
            depth: 0.4,
            color: Color4::new(1.0, 1.0, 1.0, 1.0),
        });
        self.lights.push(Light {
            position: Point3::new(player.pos.x, player.pos.y, 50.0),
            radius: 600.0,
            angle: player.dir.y.atan2(player.dir.x),
            angle_size: std::f32::consts::PI / 5.0,
            start: 18.0,
            back_glow: 30.0,
            color: Color3::from_u8(200, 200, 200).to_linear().scale(2.0),
        });
    }

    fn render_floor(&mut self, state: &State) {
        self.translucent_color_batch.push(ColorRect {
            rect: state.floor_rect(),
            color: Color4::new(0.6, 0.6, 0.6, 1.0),
            z: 0.9,
        });
    }

    fn render_wall(&mut self, wall: &Wall, visible_rect: Rect, light_rect: Rect) {
        if geom::rect_rect_overlap(light_rect, wall.rect()).is_some() {
            self.occluder_batch.push(OccluderRect {
                rect: wall.rect(),
                height: 1000.0,
                ignore_light_index1: None,
                ignore_light_index2: None,
            });
            if geom::rect_rect_overlap(visible_rect, wall.rect()).is_some() {
                self.reflector_color_batch.push(ColorRect {
                    rect: wall.rect(),
                    z: 0.2,
                    color: Color4::new(0.48, 0.48, 0.48, 1.0),
                });
            }
        }
    }

    fn render_enemy(&mut self, enemy: &Enemy, visible_rect: Rect, light_rect: Rect) {
        if enemy.dead {
            return;
        }

        let color = Color3::from_u8(170, 67, 68);

        if geom::rect_circle_overlap(light_rect, enemy.circle()).is_some() {
            /*self.circle_instances.push(ColorInstance {
                position: enemy.pos,
                angle: enemy.angle,
                color: color.to_color4(),
                z: 0.3,
                ..ColorInstance::default()
            });*/
            self.occluder_batch.push(OccluderCircle {
                circle: enemy.circle(),
                angle: 0.0,
                num_segments: 16,
                height: 200.0,
                ignore_light_index1: Some(self.lights.len() as u32),
                ignore_light_index2: None,
            });
            if geom::rect_circle_overlap(visible_rect, enemy.circle()).is_some() {
                self.reflector_sprite_batch.push(RotatedSprite {
                    rect: RotatedRect {
                        center: enemy.pos,
                        size: 2.0 * ENEMY_RADIUS * (1.0 + enemy.bump) * Vector2::new(1.0, 1.0),
                        angle: enemy.angle,
                    },
                    depth: 0.8,
                    color: Color4::new(1.0, 1.0, 1.0, 1.0),
                    tex_rect: Rect::from_top_left(
                        Point2::origin(),
                        self.enemy_texture.size().cast(),
                    ),
                });
                self.reflector_color_batch2.push(ColorCircle {
                    circle: enemy.circle(),
                    angle: enemy.angle,
                    depth: 0.8,
                    num_segments: 16,
                    color: color.to_color4(),
                });
            }
        }

        let light_circle = Circle {
            center: Point2::new(enemy.pos.x, enemy.pos.y),
            radius: 300.0,
        };
        if geom::rect_circle_overlap(visible_rect, light_circle).is_some() {
            self.lights.push(Light {
                position: Point3::new(enemy.pos.x, enemy.pos.y, 50.0),
                radius: light_circle.radius,
                angle: enemy.angle,
                angle_size: std::f32::consts::PI / 3.0,
                start: enemy.circle().radius,
                back_glow: 10.0,
                color: Color3::from_u8(200, 240, 200).to_linear().scale(0.5),
            });
        }
    }

    fn render_ball(&mut self, ball: &Ball, visible_rect: Rect, light_rect: Rect) {
        let color = Color3::from_u8(134, 187, 189);

        if geom::rect_circle_overlap(light_rect, ball.circle()).is_some() {
            self.occluder_batch.push(OccluderCircle {
                circle: ball.circle(),
                angle: 0.0,
                num_segments: 32,
                height: 1000.0,
                ignore_light_index1: None,
                ignore_light_index2: None,
            });
            if geom::rect_circle_overlap(visible_rect, ball.circle()).is_some() {
                self.reflector_color_batch.push(ColorCircle {
                    circle: ball.circle(),
                    angle: 0.0,
                    depth: 0.3,
                    num_segments: 64,
                    color: color.to_color4(),
                });
            }
        }
    }

    fn render_lamp(&mut self, lamp: &Lamp, visible_rect: Rect) {
        let color = Color3::from_u8(254, 196, 127);

        if geom::rect_circle_overlap(visible_rect, lamp.circle()).is_some() {
            self.reflector_color_batch.push(ColorCircle {
                circle: lamp.circle(),
                angle: 0.0,
                depth: 0.1,
                num_segments: 64,
                color: color.to_color4(),
            });
        }

        let light_circle = Circle {
            center: Point2::new(lamp.pos.x, lamp.pos.y),
            radius: 300.0,
        };
        if geom::rect_circle_overlap(visible_rect, light_circle).is_some() {
            self.lights.push(Light {
                position: Point3::new(lamp.pos.x, lamp.pos.y, 100.0),
                radius: light_circle.radius,
                angle: lamp.light_angle,
                angle_size: std::f32::consts::PI * 2.0,
                start: 0.0,
                back_glow: 25.0,
                color: color.to_linear().scale(0.3),
            });
        }
    }

    fn render_laser(&mut self, laser: &Laser) {
        let color = Color3::from_u8(255, 100, 100);
        /*self.reflector_color_batch.push(ColorRotatedRect {
            rect: laser.rotated_rect(),
            depth: 0.2,
            color: color.to_color4(),
        });*/
        self.source_color_batch.push(ColorRotatedRect {
            rect: laser.rotated_rect(),
            depth: 0.2,
            color: color.to_color4(),
        });
    }

    pub fn draw(
        &mut self,
        context: &Context,
        indirect_light: bool,
        blur: bool,
        debug_mode: u32,
        debug_mipmap: u32,
    ) -> Result<(), FrameError> {
        profile!("draw");

        let light = true;

        if light {
            let phase = self
                .light_pipeline
                .geometry_phase(&self.camera_matrices)?
                .draw_colors(
                    &self.translucent_light_props,
                    self.translucent_color_batch.draw_unit(),
                    &DrawParams {
                        depth_test: Some(DepthTest::default()),
                        ..DrawParams::default()
                    },
                )
                .draw_colors(
                    &self.reflector_light_props,
                    self.reflector_color_batch.draw_unit(),
                    &DrawParams {
                        depth_test: Some(DepthTest::default()),
                        ..DrawParams::default()
                    },
                )
                .draw_sprites_with_normals(
                    &self.reflector_light_props,
                    &self.enemy_texture,
                    &self.enemy_normal_texture,
                    self.reflector_sprite_batch.draw_unit(),
                    &DrawParams {
                        blend: Some(Blend::default()),
                        depth_test: Some(DepthTest::default()),
                        ..DrawParams::default()
                    },
                )
                .draw_sprites_with_normals(
                    &self.smoke_light_props,
                    &self.smoke_texture,
                    &self.smoke_normal_texture,
                    self.smoke_batch.draw_unit(),
                    &DrawParams {
                        blend: Some(Blend::default()),
                        ..DrawParams::default()
                    },
                )
                .draw_colors(
                    &self.reflector_light_props,
                    self.source_color_batch.draw_unit(),
                    &DrawParams {
                        depth_test: None,
                        ..DrawParams::default()
                    },
                )
                .shadow_map_phase(&self.lights)
                .draw_occluders(&mut self.occluder_batch)
                .build_screen_light(GlobalLightProps {
                    ambient: Color3::new(1.0, 1.0, 1.0).scale(0.08).to_linear().into(),
                    debug_mode,
                    debug_mipmap,
                    ..GlobalLightProps::default()
                })?;

            if indirect_light {
                phase
                    .indirect_light_phase()
                    .draw_color_reflectors(
                        self.reflector_color_batch.draw_unit(),
                        &DrawParams::default(),
                    )
                    .draw_color_reflectors(
                        self.reflector_color_batch2.draw_unit(),
                        &DrawParams::default(),
                    )
                    .draw_sprite_reflectors(
                        &self.smoke_texture,
                        self.smoke_batch.draw_unit(),
                        &DrawParams {
                            blend: Some(Blend::default()),
                            ..DrawParams::default()
                        },
                    )
                    .draw_color_sources(self.source_color_batch.draw_unit())
                    .prepare_cone_tracing(blur)?
                    .compose();
            } else {
                phase.compose();
            }
        } else {
            context.color_pass().draw(
                &self.camera_matrices,
                self.translucent_color_batch.draw_unit(),
                &DrawParams {
                    depth_test: Some(DepthTest::default()),
                    ..DrawParams::default()
                },
            );
            context.color_pass().draw(
                &self.camera_matrices,
                self.reflector_color_batch.draw_unit(),
                &DrawParams {
                    depth_test: Some(DepthTest::default()),
                    ..DrawParams::default()
                },
            );
            context.sprite_pass().draw(
                &self.camera_matrices,
                &self.smoke_texture,
                self.smoke_batch.draw_unit(),
                &DrawParams {
                    blend: Some(Blend::default()),
                    ..DrawParams::default()
                },
            );
        }

        context.color_pass().draw(
            &self.camera_matrices,
            self.outline_batch.draw_unit(),
            &DrawParams::default(),
        );

        self.font.draw(&self.screen_matrices, &mut self.text_batch);

        Ok(())
    }

    pub fn draw_debug_textures(&self, context: &mut Context) -> Result<(), FrameError> {
        let width = 320.0;
        let height = 200.0;
        let size = Vector2::new(width, height);

        context.draw_debug_texture(
            Rect::from_top_left(Point2::new(10.0, 10.0), size),
            &self.light_pipeline.shadow_map(),
        )?;
        context.draw_debug_texture(
            Rect::from_top_left(Point2::new(10.0, height + 20.0), size),
            &self.light_pipeline.screen_albedo(),
        )?;
        context.draw_debug_texture(
            Rect::from_top_left(Point2::new(10.0, 2.0 * height + 30.0), size),
            &self.light_pipeline.screen_normals(),
        )?;
        context.draw_debug_texture(
            Rect::from_top_left(Point2::new(10.0, 3.0 * height + 40.0), size),
            &self.light_pipeline.screen_occlusion(),
        )?;
        context.draw_debug_texture(
            Rect::from_top_left(Point2::new(10.0, 4.0 * height + 50.0), size),
            &self.light_pipeline.screen_light(),
        )?;
        context.draw_debug_texture(
            Rect::from_top_left(Point2::new(10.0, 5.0 * height + 60.0), size),
            &self.light_pipeline.screen_reflector(),
        )?;

        Ok(())
    }
}
