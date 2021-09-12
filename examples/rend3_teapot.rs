use std::fs;

use glam::{UVec2, Vec3};
use hxa::{Hxa, LayerData, NodeContent, NodeType};

fn create_mesh() -> rend3::types::Mesh {
    let data = fs::read("examples/teapot.hxa").unwrap();
    let hxa = Hxa::new(&data).unwrap();
    let node = &hxa.nodes[0];
    assert_eq!(node.type_, NodeType::Geometry);
    let content = node.content.as_ref().unwrap();
    if let NodeContent::Geometry(geometry) = content {
        let vertex_layer = &geometry.vertex_stack.layers[hxa::HC_BASE_VERTEX_LAYER_ID];
        assert_eq!(vertex_layer.name, hxa::HC_BASE_VERTEX_LAYER_NAME);
        assert_eq!(
            vertex_layer.component_count,
            hxa::HC_BASE_VERTEX_LAYER_COMPONENTS
        );

        let index_layer = &geometry.corner_stack.layers[hxa::HC_BASE_CORNER_LAYER_ID];
        assert_eq!(index_layer.name, hxa::HC_BASE_CORNER_LAYER_NAME);
        assert_eq!(
            index_layer.component_count,
            hxa::HC_BASE_CORNER_LAYER_COMPONENTS
        );

        if let LayerData::Double(vertices) = &vertex_layer.data {
            let mb = rend3::types::MeshBuilder::new(
                vertices
                    .chunks_exact(3)
                    .map(|vertex| {
                        Vec3::new(
                            vertex[0].round() as f32,
                            vertex[1].round() as f32,
                            vertex[2].round() as f32,
                        )
                    })
                    .collect(),
            );
            if let LayerData::Int32(indices) = &index_layer.data {
                let mut ngon_start = 0;
                let mut cursor = 0;
                let mut triangulated = Vec::new();
                loop {
                    let idx = indices[cursor];
                    if idx.is_negative() {
                        let vertex_count = cursor - ngon_start + 1;
                        if vertex_count < 3 {
                            panic!(
                                "A polygon must have 3 or more vertices. This one has {}",
                                vertex_count
                            )
                        } else if vertex_count == 3 {
                            triangulated.extend([
                                -indices[ngon_start + 2] as u32 - 1,
                                indices[ngon_start + 1] as u32,
                                indices[ngon_start + 0] as u32,
                            ]);
                        } else if vertex_count == 4 {
                            triangulated.extend([
                                indices[ngon_start + 2] as u32,
                                indices[ngon_start + 1] as u32,
                                indices[ngon_start + 0] as u32,
                                -indices[ngon_start + 3] as u32 - 1,
                                indices[ngon_start + 2] as u32,
                                indices[ngon_start + 0] as u32,
                            ])
                        } else {
                            panic!(
                                "Cannot handle polygons with more than 4 vertices. This one has {}",
                                vertex_count
                            );
                        }
                        ngon_start = cursor + 1;
                    }

                    cursor += 1;
                    if cursor == indices.len() {
                        assert_eq!(cursor - ngon_start, 0);
                        break;
                    }
                    if cursor - ngon_start > 4 {}
                }

                return mb.with_indices(triangulated).build();
            }
        }
    }
    unreachable!()
}

fn main() {
    env_logger::init();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = {
        let mut builder = winit::window::WindowBuilder::new();
        builder = builder.with_title("rend3 + HxA = teapot");
        builder.build(&event_loop).expect("Could not build window")
    };

    let window_size = window.inner_size();

    let iad = pollster::block_on(rend3::create_iad(None, None, None)).unwrap();

    let surface = unsafe { iad.instance.create_surface(&window) };
    let format = surface.get_preferred_format(&iad.adapter).unwrap();
    rend3::configure_surface(
        &surface,
        &iad.device,
        format,
        UVec2::new(window_size.width, window_size.height),
        rend3::types::PresentMode::Mailbox,
    );

    let renderer = rend3::Renderer::new(
        iad,
        Some(window_size.width as f32 / window_size.height as f32),
    )
    .unwrap();

    let mut routine = rend3_pbr::PbrRenderRoutine::new(
        &renderer,
        rend3_pbr::RenderTextureOptions {
            resolution: UVec2::new(window_size.width, window_size.height),
            samples: rend3_pbr::SampleCount::Four,
        },
        format,
    );

    let mesh = create_mesh();

    let mesh_handle = renderer.add_mesh(mesh);

    let material = rend3::types::Material {
        albedo: rend3::types::AlbedoComponent::Value(glam::Vec4::new(0.0, 0.5, 0.5, 1.0)),
        ..rend3::types::Material::default()
    };
    let material_handle = renderer.add_material(material);

    let object = rend3::types::Object {
        mesh: mesh_handle,
        material: material_handle,
        transform: glam::Mat4::IDENTITY,
    };
    let _object_handle = renderer.add_object(object);

    renderer.set_camera_data(rend3::types::Camera {
        projection: rend3::types::CameraProjection::Projection {
            vfov: 60.0,
            near: 0.1,
            pitch: 0.5,
            yaw: -0.55,
        },
        location: glam::Vec3A::new(9.0, 7.0, -12.0),
    });

    let _directional_handle = renderer.add_directional_light(rend3::types::DirectionalLight {
        color: glam::Vec3::ONE,
        intensity: 10.0,
        direction: glam::Vec3::new(-1.0, -4.0, 2.0),
        distance: 400.0,
    });

    event_loop.run(move |event, _, control| match event {
        winit::event::Event::WindowEvent {
            event: winit::event::WindowEvent::CloseRequested,
            ..
        } => {
            *control = winit::event_loop::ControlFlow::Exit;
        }
        winit::event::Event::WindowEvent {
            event: winit::event::WindowEvent::Resized(size),
            ..
        } => {
            let size = UVec2::new(size.width, size.height);
            rend3::configure_surface(
                &surface,
                &renderer.device,
                format,
                UVec2::new(size.x, size.y),
                rend3::types::PresentMode::Mailbox,
            );
            renderer.set_aspect_ratio(size.x as f32 / size.y as f32);
            routine.resize(
                &renderer,
                rend3_pbr::RenderTextureOptions {
                    resolution: size,
                    samples: rend3_pbr::SampleCount::One,
                },
            );
        }
        winit::event::Event::MainEventsCleared => {
            let frame = rend3::util::output::OutputFrame::from_surface(&surface).unwrap();
            let _stats = renderer.render(&mut routine, frame);
        }
        _ => {}
    });
}
