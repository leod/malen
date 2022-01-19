use coarse_prof::profile;
use nalgebra::{Matrix3, Point2, Point3, Vector2};

use malen::{
    data::{
        ColorCircle, ColorRect, ColorRotatedRect, ColorTriangleBatch, ColorVertex, InstanceBatch,
        Mesh, Sprite, SpriteBatch, TriangleTag,
    },
    geom::{Circle, Rect, Screen},
    gl::{Texture, TextureParams, Uniform},
    light::{
        GlobalLightParams, IndirectLightPipelineParams, Light, LightPipeline, LightPipelineParams,
        ObjectLightParams, OccluderBatch, OccluderCircle, OccluderRect, OccluderRotatedRect,
    },
    pass::{ColorInstance, MatricesBlock},
    text::{Font, TextBatch},
    Color3, Color4, Context, FrameError, InitError,
};

use crate::state::{Ball, Enemy, Lamp, Laser, Player, State, Wall};

pub struct Draw {
    font: Font,
    floor_texture: Texture,
    floor_normal_map: Texture,
    wall_texture: Texture,
    wall_normal_map: Texture,

    light_pipeline: LightPipeline,

    floor_light_params: Uniform<ObjectLightParams>,
    color_light_params: Uniform<ObjectLightParams>,
    wall_light_params: Uniform<ObjectLightParams>,
    camera_matrices: Uniform<MatricesBlock>,
    screen_matrices: Uniform<MatricesBlock>,

    floor_batch: SpriteBatch,
    wall_batch: SpriteBatch,
    circle_instances: InstanceBatch<ColorVertex, ColorInstance>,
    color_triangle_batch: ColorTriangleBatch,
    indirect_color_triangle_batch: ColorTriangleBatch,
    occluder_batch: OccluderBatch,
    lights: Vec<Light>,
    text_batch: TextBatch,
}

impl Draw {
    pub async fn new(context: &Context, _: &State) -> Result<Draw, InitError> {
        let font = Font::load(context, "resources/RobotoMono-Regular.ttf", 40.0).await?;
        let floor_texture = Texture::load(
            context.gl(),
            "resources/Ground_03.png",
            TextureParams::mipmapped(),
        )
        .await?;
        let floor_normal_map = Texture::load(
            context.gl(),
            "resources/Ground_03_Nrm.png",
            TextureParams::mipmapped(),
        )
        .await?;
        let wall_texture = Texture::load(
            context.gl(),
            "resources/boxesandcrates/1.png",
            TextureParams::mipmapped(),
        )
        .await?;
        let wall_normal_map = Texture::load(
            context.gl(),
            "resources/boxesandcrates/1_N.png",
            TextureParams::mipmapped(),
        )
        .await?;

        let light_pipeline = LightPipeline::new(
            context,
            LightPipelineParams {
                shadow_map_resolution: 2048,
                max_num_lights: 300,
                indirect_light: IndirectLightPipelineParams {
                    num_tracing_cones: 6,
                    num_tracing_steps: 5,
                },
            },
        )?;

        let floor_light_params = Uniform::new(
            context.gl(),
            ObjectLightParams {
                ambient_scale: 1.0,
                occlusion: 0.0,
            },
        )?;
        let color_light_params = Uniform::new(
            context.gl(),
            ObjectLightParams {
                ambient_scale: 0.95,
                occlusion: 1.0,
            },
        )?;
        let wall_light_params = Uniform::new(
            context.gl(),
            ObjectLightParams {
                ambient_scale: 0.3,
                occlusion: 1.0,
            },
        )?;
        let camera_matrices = Uniform::new(context.gl(), MatricesBlock::default())?;
        let screen_matrices = Uniform::new(context.gl(), MatricesBlock::default())?;

        let circle_mesh = Mesh::from_geometry::<TriangleTag, _>(
            context.gl(),
            ColorCircle {
                circle: Circle {
                    center: Point2::origin(),
                    radius: 20.0,
                },
                z: 0.0,
                angle: 0.0,
                num_segments: 64,
                color: Color4::new(1.0, 1.0, 1.0, 1.0),
            },
        )?;

        let circle_instances = InstanceBatch::from_mesh(circle_mesh)?;
        let color_triangle_batch = ColorTriangleBatch::new(context.gl())?;
        let indirect_color_triangle_batch = ColorTriangleBatch::new(context.gl())?;
        let floor_batch = SpriteBatch::new(context.gl())?;
        let wall_batch = SpriteBatch::new(context.gl())?;
        let occluder_batch = light_pipeline.new_occluder_batch()?;
        let lights = Vec::new();
        let text_batch = TextBatch::new(context.gl())?;

        Ok(Draw {
            font,
            floor_texture,
            floor_normal_map,
            wall_texture,
            wall_normal_map,
            light_pipeline,
            floor_light_params,
            color_light_params,
            wall_light_params,
            camera_matrices,
            screen_matrices,
            floor_batch,
            wall_batch,
            circle_instances,
            color_triangle_batch,
            indirect_color_triangle_batch,
            occluder_batch,
            lights,
            text_batch,
        })
    }

    pub fn render(&mut self, screen: Screen, state: &State) -> Result<(), FrameError> {
        profile!("Draw::render");

        self.camera_matrices.set(MatricesBlock {
            view: state.camera().matrix(screen),
            projection: screen.orthographic_projection(),
        });
        self.screen_matrices.set(MatricesBlock {
            view: Matrix3::identity(),
            projection: screen.orthographic_projection(),
        });

        self.floor_batch.clear();
        self.wall_batch.clear();
        self.circle_instances.clear();
        self.color_triangle_batch.clear();
        self.indirect_color_triangle_batch.clear();
        self.text_batch.clear();
        self.occluder_batch.clear();
        self.lights.clear();

        self.render_floor(state);
        for wall in &state.walls {
            self.render_wall(wall);
        }
        for lamp in &state.lamps {
            self.render_lamp(lamp);
        }
        for enemy in &state.enemies {
            self.render_enemy(enemy);
        }
        for ball in &state.balls {
            self.render_ball(ball);
        }
        for laser in &state.lasers {
            self.render_laser(laser);
        }
        self.render_player(&state.player);

        Ok(())
    }

    fn render_floor(&mut self, state: &State) {
        self.floor_batch.push(Sprite {
            rect: state.floor_rect(),
            tex_rect: Rect::from_top_left(
                Point2::origin(),
                self.floor_texture.size().cast::<f32>() * 20.0,
            ),
            z: 0.8,
        });
    }

    fn render_wall(&mut self, wall: &Wall) {
        if wall.use_texture {
            self.wall_batch.push(Sprite {
                rect: wall.rect(),
                z: 0.2,
                tex_rect: Rect::from_top_left(
                    Point2::origin(),
                    (wall.rect().size / 50.0)
                        .component_mul(&self.wall_texture.size().cast::<f32>()),
                ),
            });
        } else {
            self.color_triangle_batch.push(ColorRect {
                rect: wall.rect(),
                z: 0.2,
                color: Color4::new(0.3, 0.8, 0.3, 1.0),
            })
        }
        self.occluder_batch.push(OccluderRect {
            rect: wall.rect(),
            ignore_light_index1: None, //wall.lamp_index.map(|index| index as u32),
            ignore_light_index2: None,
        });
    }

    fn render_enemy(&mut self, enemy: &Enemy) {
        let color = Color3::from_u8(240, 101, 67).to_linear();
        /*self.circle_instances.push(ColorInstance {
            position: enemy.pos,
            angle: enemy.angle,
            color: color.to_color4(),
            z: 0.3,
            ..ColorInstance::default()
        });*/
        self.color_triangle_batch.push(ColorCircle {
            circle: enemy.circle(),
            angle: enemy.angle,
            z: 0.3,
            num_segments: 16,
            color: color.to_color4(),
        });
        self.occluder_batch.push(OccluderCircle {
            circle: enemy.circle(),
            angle: 0.0,
            num_segments: 16,
            ignore_light_index1: Some(self.lights.len() as u32),
            ignore_light_index2: None,
        });
        self.lights.push(Light {
            position: Point3::new(enemy.pos.x, enemy.pos.y, 50.0),
            radius: 300.0,
            angle: enemy.angle,
            angle_size: std::f32::consts::PI / 3.0,
            start: 20.0,
            color: Color3::from_u8(212, 230, 135).to_linear().scale(0.3),
        });
    }

    fn render_ball(&mut self, ball: &Ball) {
        let color = Color3::from_u8(134, 187, 189).to_linear();
        self.color_triangle_batch.push(ColorCircle {
            circle: ball.circle(),
            angle: 0.0,
            z: 0.3,
            num_segments: 64,
            color: color.to_color4(),
        });
        self.occluder_batch.push(OccluderCircle {
            circle: ball.circle(),
            angle: 0.0,
            num_segments: 32,
            ignore_light_index1: None,
            ignore_light_index2: None,
        });
    }

    fn render_lamp(&mut self, lamp: &Lamp) {
        let color = Color3::from_u8(254, 196, 127).to_linear();
        self.color_triangle_batch.push(ColorCircle {
            circle: lamp.circle(),
            angle: 0.0,
            z: 0.1,
            num_segments: 64,
            color: color.to_color4(),
        });
        self.lights.push(Light {
            position: Point3::new(lamp.pos.x, lamp.pos.y, 100.0),
            radius: 300.0,
            angle: lamp.light_angle,
            angle_size: std::f32::consts::PI * 2.0,
            start: 0.0,
            color: color.scale(0.7),
        });
    }

    fn render_laser(&mut self, laser: &Laser) {
        let color = Color3::from_u8(200, 70, 30).to_linear();
        self.color_triangle_batch.push(ColorRotatedRect {
            rect: laser.rotated_rect(),
            z: 0.2,
            color: color.to_color4(),
        });
        self.indirect_color_triangle_batch.push(ColorRotatedRect {
            rect: laser.rotated_rect(),
            z: 0.2,
            color: color.scale(0.5).to_color4(),
        });
    }

    fn render_player(&mut self, player: &Player) {
        let color = Color3::from_u8(255, 209, 102).to_linear();
        self.color_triangle_batch.push(ColorRotatedRect {
            rect: player.rotated_rect(),
            z: 0.4,
            color: color.to_color4(),
        });
        self.occluder_batch.push(OccluderRotatedRect {
            rect: player.rotated_rect(),
            ignore_light_index1: Some(self.lights.len() as u32),
            ignore_light_index2: Some(self.lights.len() as u32 + 1),
        });
        self.lights.push(Light {
            position: Point3::new(player.pos.x, player.pos.y, 50.0),
            radius: 600.0,
            angle: player.dir.y.atan2(player.dir.x),
            angle_size: std::f32::consts::PI / 6.0,
            start: 18.0,
            color: Color3::from_u8(255, 255, 255).to_linear(),
        });
        /*self.lights.push(Light {
            position: Point3::new(player.pos.x, player.pos.y, 50.0),
            radius: 40.0,
            angle: player.angle,
            angle_size: std::f32::consts::PI * 2.0,
            start: 0.0,
            color: Color3::from_u8(150, 150, 150).to_linear(),
        });*/
    }

    pub fn draw(&mut self, indirect_light: bool) -> Result<(), FrameError> {
        profile!("Draw::draw");

        let phase = self
            .light_pipeline
            .geometry_phase(&self.camera_matrices)?
            .draw_colors(
                &self.color_light_params,
                self.color_triangle_batch.draw_unit(),
            )
            .draw_sprites_with_normals(
                &self.floor_light_params,
                &self.floor_texture,
                &self.floor_normal_map,
                self.floor_batch.draw_unit(),
            )?
            .draw_sprites_with_normals(
                &self.wall_light_params,
                &self.wall_texture,
                &self.wall_normal_map,
                self.wall_batch.draw_unit(),
            )?
            .shadow_map_phase(&self.lights)
            .draw_occluders(&mut self.occluder_batch)
            .build_screen_light(GlobalLightParams {
                ambient: Color3::new(1.0, 1.0, 1.0).scale(0.13).to_linear(),
                ..GlobalLightParams::default()
            });

        if indirect_light {
            phase
                .indirect_light_phase()
                .draw_color_reflectors(self.color_triangle_batch.draw_unit())
                .draw_sprite_reflectors(&self.wall_texture, self.wall_batch.draw_unit())?
                .draw_color_sources(self.indirect_color_triangle_batch.draw_unit())
                .prepare_cone_tracing()
                .compose();
        } else {
            phase.compose();
        }

        self.font
            .draw(&self.screen_matrices, &mut self.text_batch)?;

        Ok(())
    }

    pub fn draw_debug_textures(&self, context: &mut Context) -> Result<(), FrameError> {
        context.draw_debug_texture(
            Rect::from_top_left(Point2::new(10.0, 10.0), Vector2::new(320.0, 240.0)),
            &self.light_pipeline.shadow_map(),
        )?;
        context.draw_debug_texture(
            Rect::from_top_left(Point2::new(10.0, 260.0), Vector2::new(320.0, 240.0)),
            &self.light_pipeline.screen_albedo(),
        )?;
        context.draw_debug_texture(
            Rect::from_top_left(Point2::new(10.0, 510.0), Vector2::new(320.0, 240.0)),
            &self.light_pipeline.screen_normals(),
        )?;
        context.draw_debug_texture(
            Rect::from_top_left(Point2::new(10.0, 760.0), Vector2::new(320.0, 240.0)),
            &self.light_pipeline.screen_occlusion(),
        )?;
        context.draw_debug_texture(
            Rect::from_top_left(Point2::new(340.0, 10.0), Vector2::new(320.0, 240.0)),
            &self.light_pipeline.screen_light(),
        )?;
        context.draw_debug_texture(
            Rect::from_top_left(Point2::new(340.0, 260.0), Vector2::new(320.0, 240.0)),
            &self.light_pipeline.screen_reflectors(),
        )?;

        Ok(())
    }
}
