use coarse_prof::profile;
use nalgebra::{Matrix3, Point2, Vector2};

use malen::{
    data::{
        ColorCircle, ColorLineBatch, ColorRotatedRect, ColorTriangleBatch, ColorVertex,
        InstanceBatch, Mesh, Sprite, SpriteBatch, TriangleTag,
    },
    geom::{Circle, Rect, Screen},
    gl::{DepthTest, DrawParams, Texture, TextureParams, UniformBuffer},
    light::{
        GlobalLightParams, Light, LightPipeline, LightPipelineParams, OccluderBatch,
        OccluderCircle, OccluderRect, OccluderRotatedRect,
    },
    pass::{ColorInstance, MatricesBlock},
    text::{Font, TextBatch},
    Color3, Color4, Context, FrameError, InitError,
};

use crate::state::{self, Ball, Enemy, Lamp, Player, State, Wall};

pub struct Draw {
    font: Font,
    texture: Texture,
    normal_map: Texture,
    texture2: Texture,
    normal_map2: Texture,

    camera_matrices: UniformBuffer<MatricesBlock>,
    screen_matrices: UniformBuffer<MatricesBlock>,

    circle_instances: InstanceBatch<ColorVertex, ColorInstance>,
    color_batch: ColorTriangleBatch,
    shaded_color_batch: ColorTriangleBatch,
    shaded_sprite_batch: SpriteBatch,
    shaded_sprite_batch2: SpriteBatch,
    outline_batch: ColorLineBatch,
    text_batch: TextBatch,

    light_pipeline: LightPipeline,
    occluder_batch: OccluderBatch,
    lights: Vec<Light>,
}

impl Draw {
    pub async fn new(context: &Context) -> Result<Draw, InitError> {
        let font = Font::load(context, "resources/RobotoMono-Regular.ttf", 40.0).await?;
        let texture = Texture::load(
            context.gl(),
            "resources/Ground_03.png",
            TextureParams::default(),
        )
        .await?;
        let normal_map = Texture::load(
            context.gl(),
            "resources/Ground_03_Nrm.png",
            TextureParams::default(),
        )
        .await?;
        let texture2 = Texture::load(
            context.gl(),
            "resources/boxesandcrates/1.png",
            TextureParams::default(),
        )
        .await?;
        let normal_map2 = Texture::load(
            context.gl(),
            "resources/boxesandcrates/1_N.png",
            TextureParams::default(),
        )
        .await?;

        let camera_matrices = UniformBuffer::new(context.gl(), MatricesBlock::default())?;
        let screen_matrices = UniformBuffer::new(context.gl(), MatricesBlock::default())?;

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
        let color_batch = ColorTriangleBatch::new(context.gl())?;
        let shaded_color_batch = ColorTriangleBatch::new(context.gl())?;
        let shaded_sprite_batch = SpriteBatch::new(context.gl())?;
        let shaded_sprite_batch2 = SpriteBatch::new(context.gl())?;
        let outline_batch = ColorLineBatch::new(context.gl())?;
        let text_batch = TextBatch::new(context.gl())?;

        let light_pipeline = LightPipeline::new(
            context,
            LightPipelineParams {
                shadow_map_resolution: 2048,
                max_num_lights: 300,
            },
        )?;
        let occluder_batch = light_pipeline.new_occluder_batch()?;
        let lights = Vec::new();

        Ok(Draw {
            font,
            texture,
            normal_map,
            texture2,
            normal_map2,
            camera_matrices,
            screen_matrices,
            circle_instances,
            color_batch,
            shaded_color_batch,
            shaded_sprite_batch,
            shaded_sprite_batch2,
            outline_batch,
            text_batch,
            light_pipeline,
            occluder_batch,
            lights,
        })
    }

    pub fn render(&mut self, screen: Screen, state: &State) -> Result<(), FrameError> {
        profile!("Draw::render");

        self.camera_matrices.set_data(MatricesBlock {
            view: state.camera().matrix(screen),
            projection: screen.orthographic_projection(),
        });
        self.screen_matrices.set_data(MatricesBlock {
            view: Matrix3::identity(),
            projection: screen.orthographic_projection(),
        });

        self.circle_instances.clear();
        self.color_batch.clear();
        self.shaded_color_batch.clear();
        self.shaded_sprite_batch.clear();
        self.shaded_sprite_batch2.clear();
        self.outline_batch.clear();
        self.text_batch.clear();
        self.occluder_batch.clear();
        self.lights.clear();

        self.render_floor();
        for lamp in &state.lamps {
            self.render_lamp(lamp);
        }
        for wall in &state.walls {
            self.render_wall(wall);
        }
        for enemy in &state.enemies {
            self.render_enemy(enemy);
        }
        for ball in &state.balls {
            self.render_ball(ball);
        }
        self.render_player(&state.player);

        Ok(())
    }

    fn render_floor(&mut self) {
        self.shaded_sprite_batch.push(Sprite {
            rect: Rect {
                center: Point2::origin(),
                size: 2.0 * Vector2::new(state::MAP_SIZE, state::MAP_SIZE),
            },
            tex_rect: Rect::from_top_left(
                Point2::origin(),
                self.texture.size().cast::<f32>() * 20.0,
            ),
            z: 0.8,
        });
    }

    fn render_wall(&mut self, wall: &Wall) {
        self.shaded_sprite_batch2.push(Sprite {
            rect: wall.rect(),
            z: 0.2,
            tex_rect: Rect::from_top_left(
                Point2::origin(),
                (wall.rect().size / 50.0).component_mul(&self.texture2.size().cast::<f32>()),
            ),
        });
        /*self.outline_batch.push(ColorRect {
            rect: wall.rect(),
            z: 0.2,
            color: Color4::new(1.0, 1.0, 1.0, 1.0),
        });*/
        self.occluder_batch.push(OccluderRect {
            rect: wall.rect(),
            color: Color3::from_u8(69, 157, 69),
            ignore_light_index: wall.lamp_index.map(|index| index as u32),
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
        self.shaded_color_batch.push(ColorCircle {
            circle: enemy.circle(),
            angle: enemy.angle,
            z: 0.3,
            num_segments: 16,
            color: color.to_color4(),
        });
        /*self.outline_batch.push(ColorCircle {
            circle: enemy.circle(),
            angle: 0.0,
            z: 0.0,
            num_segments: 16,
            color: Color4::from_u8(255, 255, 255, 255),
        });*/
        self.occluder_batch.push(OccluderCircle {
            circle: enemy.circle(),
            angle: 0.0,
            num_segments: 16,
            color: color,
            ignore_light_index: Some(self.lights.len() as u32),
        });
        self.lights.push(Light {
            position: enemy.pos,
            radius: 500.0,
            angle: enemy.angle,
            angle_size: std::f32::consts::PI / 3.0,
            start: 18.0,
            color: color.scale(4.0),
        });
    }

    fn render_ball(&mut self, ball: &Ball) {
        let color = Color3::from_u8(134, 187, 189).to_linear();
        self.shaded_color_batch.push(ColorCircle {
            circle: ball.circle(),
            angle: 0.0,
            z: 0.3,
            num_segments: 64,
            color: color.to_color4(),
        });
        /*self.outline_batch.push(ColorCircle {
            circle: ball.circle(),
            angle: 0.0,
            z: 0.0,
            num_segments: 64,
            color: Color4::from_u8(255, 255, 255, 255),
        });*/
        self.occluder_batch.push(OccluderCircle {
            circle: ball.circle(),
            angle: 0.0,
            num_segments: 16,
            color: color,
            ignore_light_index: None,
        });
    }

    fn render_lamp(&mut self, lamp: &Lamp) {
        let color = Color3::from_u8(254, 196, 127).to_linear();
        self.shaded_color_batch.push(ColorCircle {
            circle: lamp.circle(),
            angle: 0.0,
            z: 0.1,
            num_segments: 64,
            color: color.to_color4(),
        });
        /*self.outline_batch.push(ColorCircle {
            circle: lamp.circle(),
            angle: 0.0,
            z: 0.0,
            num_segments: 64,
            color: Color4::from_u8(255, 255, 255, 255),
        });*/
        self.lights.push(Light {
            position: lamp.pos,
            radius: 300.0,
            angle: lamp.light_angle,
            angle_size: std::f32::consts::PI * 2.0,
            start: 0.0,
            color: color.scale(7.0),
        });
    }

    fn render_player(&mut self, player: &Player) {
        let color = Color3::from_u8(255, 209, 102).to_linear();
        self.shaded_color_batch.push(ColorRotatedRect {
            rect: player.rotated_rect(),
            z: 0.4,
            color: color.to_color4(),
        });
        self.outline_batch.push(ColorRotatedRect {
            rect: player.rotated_rect(),
            z: 0.4,
            color: Color4::new(1.0, 1.0, 1.0, 1.0),
        });
        self.occluder_batch.push(OccluderRotatedRect {
            rect: player.rotated_rect(),
            color: color,
            ignore_light_index: Some(self.lights.len() as u32),
        });
        self.lights.push(Light {
            position: player.pos,
            radius: 1200.0,
            angle: player.angle,
            angle_size: std::f32::consts::PI / 6.0,
            start: 22.0,
            color: Color3::from_u8(150, 150, 150).to_linear().scale(30.0),
        });
        /*self.lights.push(Light {
            position: self.state.player.pos,
            radius: 100.0,
            angle: self.state.player.angle,
            angle_size: 2.0 * std::f32::consts::PI,
            color: Color3::new(0.8, 0.8, 2.0),
        });*/
    }

    pub fn draw(&mut self, context: &mut Context, show_textures: bool) -> Result<(), FrameError> {
        profile!("Draw::draw");

        self.light_pipeline
            .geometry_phase(&self.camera_matrices)?
            .draw_geometry_colors(self.shaded_color_batch.draw_unit())
            .draw_geometry_sprite_normals(
                &self.texture,
                &self.normal_map,
                self.shaded_sprite_batch.draw_unit(),
            )?
            .draw_geometry_sprite_normals(
                &self.texture2,
                &self.normal_map2,
                self.shaded_sprite_batch2.draw_unit(),
            )?
            .shadow_map_phase(&self.lights)
            .draw_occluders(&mut self.occluder_batch)
            .build_screen_light()
            .compose(GlobalLightParams {
                ambient: Color3::new(0.004, 0.004, 0.004),
            });

        /*context.color_pass().draw(
            &self.camera_matrices,
            self.color_batch.draw_unit(),
            &DrawParams {
                depth_test: Some(DepthTest::default()),
                ..DrawParams::default()
            },
        );*/
        context.instanced_color_pass().draw(
            &self.camera_matrices,
            self.circle_instances.draw_unit(),
            &DrawParams {
                depth_test: Some(DepthTest::default()),
                ..DrawParams::default()
            },
        );

        context.color_pass().draw(
            &self.camera_matrices,
            self.outline_batch.draw_unit(),
            &DrawParams::default(),
        );

        self.font
            .draw(&self.screen_matrices, &mut self.text_batch)?;

        if show_textures {
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
                &self.light_pipeline.screen_light(),
            )?;
        }

        Ok(())
    }
}
