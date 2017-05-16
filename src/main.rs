// TODO: This file is utter mess, should be refactored

#![feature(collections)]

#[macro_use]
extern crate glium;
extern crate collections;
#[macro_use]
extern crate itertools;
extern crate nalgebra as na;
extern crate cam;
extern crate vecmath;
extern crate rand;
extern crate time;

mod marching_cubes_data;
mod linspace;

use glium::{DisplayBuild, Surface};
use glium::backend::glutin_backend::GlutinFacade;
use linspace::linspace;
use glium::texture::buffer_texture::BufferTexture;
use glium::texture::buffer_texture::BufferTextureType;
use glium::glutin::Event;
use glium::glutin::ElementState;
use glium::glutin::VirtualKeyCode;
use cam::{Camera, CameraPerspective};
use glium::texture::texture2d::Texture2d;
use glium::texture::RawImage2d;
use rand::{SeedableRng, XorShiftRng, Rng};
use std::cmp;

const MAX_METABALLS_SIZE: usize = 64;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
}
implement_vertex!(Vertex, position);

impl Default for Vertex {
    fn default() -> Self {
        Vertex { position: [0.0f32, 0.0, 0.0] }
    }
}
#[derive(Copy, Clone)]
struct BorderVertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
}
implement_vertex!(BorderVertex, position, tex_coord);

impl BorderVertex {
    fn new(pos: [f32; 3], tex: [f32; 2]) -> BorderVertex {
        BorderVertex {
            position: pos,
            tex_coord: tex,
        }
    }
}

fn create_tri_table_texture(display: &GlutinFacade) -> BufferTexture<(i8, i8, i8, i8)> {
    BufferTexture::immutable(display,
                             &marching_cubes_data::tri_table(),
                             BufferTextureType::Integral)
        .expect("could not create triangle table buffer")
}

fn load_shaders(display: &GlutinFacade) -> (glium::Program, glium::Program) {

    let vertex_shader_src = include_str!("Shaders/metaball_vertex.glsl");
    let geometry_shader_src = include_str!("Shaders/metaball_geometry.glsl");
    let fragment_shader_src = include_str!("Shaders/metaball_fragment.glsl");
    let metaball_program = glium::Program::from_source(display,
                                                       vertex_shader_src,
                                                       fragment_shader_src,
                                                       Some(geometry_shader_src))
                               .expect("couldn't create border program");

    let border_vertex_shader = include_str!("Shaders/border_vertex.glsl");
    let border_fragment_shader = include_str!("Shaders/border_fragment.glsl");
    let border_program = glium::Program::from_source(display,
                                                     border_vertex_shader,
                                                     border_fragment_shader,
                                                     None)
                             .expect("couldn't create border program");

    (metaball_program, border_program)
}

fn get_grid(a: f32, b: f32, resolution: usize) -> Vec<Vertex> {
    iproduct!(linspace(a, b, resolution),
              linspace(a, b, resolution),
              linspace(a, b, resolution))
        .map(|(x, y, z)| Vertex { position: [x, y, z] })
        .collect()
}

fn get_border_vertices(start: f32, end: f32) -> Vec<BorderVertex> {
    let mut res = Vec::with_capacity(36);
    let v = [(start, start, start),
             (end, start, start),
             (end, end, start),
             (start, end, start),
             (start, start, end),
             (end, start, end),
             (end, end, end),
             (start, end, end)];
    for face in 0..6 {
        let _00 = match face {
            0 => v[0],
            1 => v[1],
            2 => v[5],
            3 => v[4],
            4 => v[4],
            5 => v[3],
            _ => unreachable!(),
        };
        let _01 = match face {
            0 => v[1],
            1 => v[5],
            2 => v[4],
            3 => v[0],
            4 => v[5],
            5 => v[2],
            _ => unreachable!(),
        };
        let _10 = match face {
            0 => v[3],
            1 => v[2],
            2 => v[6],
            3 => v[7],
            4 => v[0],
            5 => v[7],
            _ => unreachable!(),
        };
        let _11 = match face {
            0 => v[2],
            1 => v[6],
            2 => v[7],
            3 => v[3],
            4 => v[1],
            5 => v[6],
            _ => unreachable!(),
        };

        res.push(BorderVertex::new([_00.0, _00.1, _00.2], [0.0, 0.0]));
        res.push(BorderVertex::new([_10.0, _10.1, _10.2], [50.0, 0.0]));
        res.push(BorderVertex::new([_01.0, _01.1, _01.2], [0.0, 50.0]));

        res.push(BorderVertex::new([_10.0, _10.1, _10.2], [50.0, 0.0]));
        res.push(BorderVertex::new([_11.0, _11.1, _11.2], [50.0, 50.0]));
        res.push(BorderVertex::new([_01.0, _01.1, _01.2], [0.0, 50.0]));
    }
    res
}

fn get_border_texture(display: &GlutinFacade) -> Texture2d {
    let (dim_x, dim_y) = (32, 32);
    let mut data = Vec::with_capacity(dim_x * dim_y);
    for y in 0..dim_y {
        for x in 0..dim_x {
            data.push(if x < dim_x / 16 || y < dim_y / 16 {
                (0u8, 0u8, 0u8, 255u8)
            } else {
                (0, 0, 0, 0)
            });
        }
    }
    let raw_tex = RawImage2d::from_raw_rgba(data, (dim_x as u32, dim_y as u32));
    Texture2d::new(display, raw_tex).expect("couldn't create border texture")
}

fn update_metaball_positions(metaballs: &mut [(f32, f32, f32, f32)], t: f32) {
    let mut rng: XorShiftRng = SeedableRng::from_seed([1, 3, 3, 7]);
    for i in 0..MAX_METABALLS_SIZE {
        let xtmul = rng.gen::<f32>();
        let xoff = rng.gen::<f32>();
        let xmul = (rng.gen::<f32>() - 0.5) * 3.5;
        let ytmul = rng.gen::<f32>();
        let yoff = rng.gen::<f32>();
        let ymul = (rng.gen::<f32>() - 0.5) * 3.5;
        let ztmul = rng.gen::<f32>();
        let zoff = rng.gen::<f32>();
        let zmul = (rng.gen::<f32>() - 0.5) * 3.5;

        metaballs[i].0 = (t * xtmul + xoff).sin() * xmul;
        metaballs[i].1 = (t * ytmul + yoff).sin() * ymul;
        metaballs[i].2 = (t * ztmul + zoff).sin() * zmul;
    }
}

fn change_resolution(display: &GlutinFacade,
                     space_start: f32,
                     space_end: f32,
                     resolution: usize)
                     -> glium::VertexBuffer<Vertex> {
    let shape = get_grid(space_start, space_end, resolution);
    glium::VertexBuffer::new(display, &shape)
        .expect("could not create the vertex buffer with the get_grid")
}

fn recalculate_step(space_start: f32, space_end: f32, resolution: usize) -> f32 {
    ((space_end - space_start) / ((resolution - 1) as f32)).abs()
}

fn main() {
    let display = glium::glutin::WindowBuilder::new()
                      .with_depth_buffer(24)
                      .build_glium()
                      .unwrap();
    let mut resolution = 100usize;
    let space_start = -2.0f32;
    let space_end = 2.0f32;
    let mut step = recalculate_step(space_start, space_end, resolution);
    println!("step: {}", step);

    let mut vertex_buffer = change_resolution(&display, space_start, space_end, resolution);
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);

    let border_texture = get_border_texture(&display);
    let border_texture =
        border_texture.sampled()
                      .wrap_function(glium::uniforms::SamplerWrapFunction::Repeat)
                      .minify_filter(glium::uniforms::MinifySamplerFilter::LinearMipmapLinear)
                      .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
                      .anisotropy(16);
    let border = get_border_vertices(space_start, space_end);
    let border_vertex_buf = glium::VertexBuffer::new(&display, &border)
                                .expect("could not create vertex buffer with borders");
    let border_indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let tri_table = create_tri_table_texture(&display);

    let (program, border_program) = load_shaders(&display);

    let mut metaballs = [(0.0f32, 0.0f32, 0.0f32, 0.0f32); MAX_METABALLS_SIZE];

    let mut metaballs_size = 0usize;
    let mut metaballs_buffer: BufferTexture<(f32, f32, f32, f32)> =
        BufferTexture::dynamic(&display, &metaballs, BufferTextureType::Float)
            .expect("couldn't create metaballs buffer");

    let dimens = display.get_framebuffer_dimensions();
    let mut camera = Camera::new([0.0f32, 0.0, 4.0]);
    let mut camera_perspective = CameraPerspective {
        fov: 45.0,
        near_clip: 0.2,
        far_clip: 1024.0,
        aspect_ratio: dimens.0 as f32 / dimens.1 as f32,
    };

    let mut t = 0.5f32;
    let mut dt = 0.0f32;

    let mut ctrl_down = false;
    let mut rmb_down = false;
    let (mut mouse_x, mut mouse_y) = (0, 0);
    let mut directionwise_move_factor = 0.0f32;
    let mut sidewise_move_factor = 0.0f32;
    let mut upwards_move_factor = 0.0f32;
    let mut pitch = 0.0f32;
    let mut yaw = 0.0f32;
    let mut now: f64;
    let mut previous = 0.0f64;
    let mut r_down = false;

    loop {
        now = time::precise_time_s();
        let delta = (now - previous) as f32;
        t += dt * delta;

        let cam_pos = camera.position;
        let cam_pos: &na::Vec3<_> = (&cam_pos).into();
        let cam_dir = camera.forward;
        let cam_dir: &na::Vec3<_> = (&cam_dir).into();
        let sideways = camera.right;
        let sideways: &na::Vec3<_> = (&sideways).into();
        let upwards = camera.up;
        let upwards: &na::Vec3<_> = (&upwards).into();
        let new_cam_pos = *cam_pos + *cam_dir * directionwise_move_factor * delta * 2.0;
        let new_cam_pos = new_cam_pos + *sideways * sidewise_move_factor * delta * 2.0;
        let new_cam_pos = new_cam_pos + *upwards * upwards_move_factor * delta * 2.0;
        camera.position = *new_cam_pos.as_ref();

        update_metaball_positions(&mut metaballs, t);

        let mut target = display.draw();
        target.clear_color_and_depth((0.1, 0.1, 0.4, 1.0), 1.0);

        let border_uniforms = uniform! {
            tex: border_texture,
            persp: camera_perspective.projection(),
            view: camera.orthogonal()
        };

        let border_params = glium::DrawParameters {
            blend: glium::Blend {
                alpha: glium::BlendingFunction::Addition {
                    source: glium::LinearBlendingFactor::SourceAlpha,
                    destination: glium::LinearBlendingFactor::OneMinusSourceAlpha,
                },
                color: glium::BlendingFunction::Addition {
                    source: glium::LinearBlendingFactor::SourceAlpha,
                    destination: glium::LinearBlendingFactor::OneMinusSourceAlpha,
                },
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
            ..Default::default()
        };

        target.draw(&border_vertex_buf,
                    &border_indices,
                    &border_program,
                    &border_uniforms,
                    &border_params)
              .unwrap();

        // this scope is important, since we don't want the metaballs_buffer to be borrowed
        // for the rest of the loop
        {
            let uniforms = uniform! {
                perspective: camera_perspective.projection(),
                metaballs: &metaballs_buffer,
                metaballsLength: metaballs_size as i32,
                view: camera.orthogonal(),
                triTableTex: &tri_table,
                cubeSideLength: step,
                isolevel: 0.5f32,
                light: (-1.0f32, 1.0f32, 1.0f32),
                eye: (new_cam_pos.x, new_cam_pos.y, new_cam_pos.z),
            };

            let params = glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                ..Default::default()
            };

            target.draw(&vertex_buffer, &indices, &program, &uniforms, &params)
                  .unwrap();
            target.finish().unwrap();
        }

        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,
                Event::Resized(w, h) => camera_perspective.aspect_ratio = w as f32 / h as f32,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::LControl)) => {
                    ctrl_down = true
                }
                Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::LControl)) => {
                    ctrl_down = false
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::R)) => {
                    r_down = true
                }
                Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::R)) => {
                    r_down = false
                }

                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::W)) => {
                    directionwise_move_factor = -1.0
                }
                Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::W)) => {
                    directionwise_move_factor = 0.0
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::S)) => {
                    directionwise_move_factor = 1.0
                }
                Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::S)) => {
                    directionwise_move_factor = 0.0
                }

                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::D)) => {
                    sidewise_move_factor = 1.0
                }
                Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::D)) => {
                    sidewise_move_factor = 0.0
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::A)) => {
                    sidewise_move_factor = -1.0
                }
                Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::A)) => {
                    sidewise_move_factor = 0.0
                }

                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Space)) => {
                    upwards_move_factor = 1.0
                }
                Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::Space)) => {
                    upwards_move_factor = 0.0
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::LShift)) => {
                    upwards_move_factor = -1.0
                }
                Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::LShift)) => {
                    upwards_move_factor = 0.0
                }

                Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::Q)) => {
                    add_random_metaball(&mut metaballs, &mut metaballs_size)
                }
                Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::E)) => {
                    remove_random_metaball(&mut metaballs_size)
                }

                Event::MouseWheel(delta) if r_down => {
                    let amt = match delta {
                        glium::glutin::MouseScrollDelta::LineDelta(_, a) => a,
                        glium::glutin::MouseScrollDelta::PixelDelta(_, a) => a,
                    };
                    println!("amt: {}", amt);
                    resolution = cmp::max(cmp::min(resolution as i32 +
                                                   amt as i32 *
                                                   if resolution < 20 {
                                                       1
                                                   } else {
                                                       10
                                                   },
                                                   200),
                                          2) as usize;
                    vertex_buffer = change_resolution(&display, space_start, space_end, resolution);
                    step = recalculate_step(space_start, space_end, resolution);
                    println!("grid resolution: {}", resolution);
                }
                Event::MouseWheel(delta) if ctrl_down => {
                    let amt = match delta {
                        glium::glutin::MouseScrollDelta::LineDelta(_, a) => a,
                        glium::glutin::MouseScrollDelta::PixelDelta(_, a) => a,
                    };
                    let fov = camera_perspective.fov;
                    camera_perspective.fov = (fov + amt).min(120.0).max(10.0);
                    println!("fov: {}", (fov + amt).min(120.0).max(10.0));
                }
                Event::MouseWheel(delta) => {
                    let amt = match delta {
                        glium::glutin::MouseScrollDelta::LineDelta(_, a) => a,
                        glium::glutin::MouseScrollDelta::PixelDelta(_, a) => a,
                    };
                    dt += amt;
                }

                Event::MouseInput(ElementState::Pressed, glium::glutin::MouseButton::Right) => {
                    rmb_down = true;
                }
                Event::MouseInput(ElementState::Released, glium::glutin::MouseButton::Right) => {
                    rmb_down = false
                }
                Event::MouseMoved((x, y)) if rmb_down => {
                    pitch = pitch - (y - mouse_y) as f32 / 1000.0;
                    pitch = pitch.min(90.0f32.to_radians()).max(-90.0f32.to_radians());
                    yaw = yaw + (x - mouse_x) as f32 / 1000.0;
                    camera.set_yaw_pitch(yaw, pitch);
                    mouse_x = x;
                    mouse_y = y;
                }
                Event::MouseMoved((x, y)) => {
                    mouse_x = x;
                    mouse_y = y;
                }
                _ => (),
            }
        }

        update_metaballs(&mut metaballs, &mut metaballs_buffer);

        previous = now;
    }
}

fn update_metaballs(metaballs: &mut [(f32, f32, f32, f32)],
                    metaballs_buffer: &mut BufferTexture<(f32, f32, f32, f32)>) {
    metaballs_buffer.write(&metaballs);
}

fn add_random_metaball(metaballs: &mut [(f32, f32, f32, f32)], metaballs_size: &mut usize) {
    if *metaballs_size == MAX_METABALLS_SIZE {
        return;
    }

    metaballs[*metaballs_size].3 = if rand::random::<f32>() > 0.1 {
        0.4 + rand::random::<f32>() * 0.4
    } else {
        -0.2 - rand::random::<f32>() * 0.4
    };

    *metaballs_size += 1;
    println!("metaballs: {}", *metaballs_size);
}

fn remove_random_metaball(metaballs_size: &mut usize) {
    if *metaballs_size == 0 {
        return;
    }
    *metaballs_size -= 1;
    println!("metaballs: {}", *metaballs_size);
}
