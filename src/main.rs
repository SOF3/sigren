use std::{fs, ops::Range, rc::Rc, cell::RefCell};

use serde::Deserialize;
use three_d::{
    core::Program, degrees, vec2, vec3, Blend, Camera, ClearState, Clip, CpuMesh,
    Cull, DepthTest, FrameOutput, Material, Model, Object, Positions, RenderStates, Screen,
    SquareMatrix, Vec3, Window, WindowSettings, WriteMask, CameraAction, OrbitControl, Axes,
};

fn main() {
    let window = Window::new(WindowSettings {
        title: "signal render".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();

    let signals = serde_yaml::from_str(&fs::read_to_string("signals.yml").unwrap()).unwrap();

    let gl = window.gl().unwrap();

    let make_camera = || Camera::new_perspective(
        &gl,
        window.viewport().unwrap(),
        vec3(0., 0., 2.),
        vec3(0., 0., 0.),
        vec3(0., 1., 0.),
        degrees(90.),
        0.1,
        100.,
    ) .unwrap();

    let init_camera = make_camera();
    let camera = Rc::new(RefCell::new(make_camera()));

    let mut camera_control = OrbitControl::new(*init_camera.target(), 1., 100.);

    let mesh = Model::new_with_material(
        &gl,
        &CpuMesh {
            positions: Positions::F32(vec![
                vec3(-1., -1., 0.,),
                vec3(1., -1., 0.,),
                vec3(1., 1., 0.,),
                vec3(-1., -1., 0.,),
                vec3(1., 1., 0.,),
                vec3(-1., 1., 0.,),
            ]),
            uvs: Some(vec![
                vec2(0., 0.),
                vec2(1., 0.),
                vec2(1., 1.),
                vec2(0., 0.),
                vec2(1., 1.),
                vec2(0., 1.),
            ]),
            ..Default::default()
        },
        SignalViewer {
            range: 0.0..100.,
            signals,
            camera: Rc::clone(&camera),
        },
    )
    .unwrap();

    let axes = Axes::new(&gl, 0.02, 1.0).unwrap();

    window
        .render_loop(move |mut frame| {
            camera_control
                .handle_events(&mut *camera.borrow_mut(), &mut frame.events)
                .unwrap();
            Screen::write(&gl, ClearState::default(), || {
                axes.render(&*camera.borrow(), &[])?;
                mesh.render(&init_camera, &[])?;
                Ok(())
            }).unwrap();
            FrameOutput {
                swap_buffers: true,
                wait_next_event: true,
                ..Default::default()
            }
        })
        .unwrap();
}

struct SignalViewer {
    signals: Vec<Signal>,
    range: Range<f32>,
    camera: Rc<RefCell<Camera>>,
}

impl Material for SignalViewer {
    fn fragment_shader_source(
        &self,
        _use_vertex_colors: bool,
        _lights: &[&dyn three_d::Light],
    ) -> String {
        fs::read_to_string("frag.glsl")
            .unwrap()
            .replace("NUM_SIGNALS", &self.signals.len().to_string())
    }

    fn use_uniforms(
        &self,
        program: &Program,
        _init_camera: &Camera,
        _lights: &[&dyn three_d::Light],
    ) -> three_d::ThreeDResult<()> {
        program.use_uniform_mat4(
            "p",
            &self.camera.borrow().view().invert().expect("camera is not invertible"),
        )?;

        let unif_k: Vec<_> = self.signals.iter().map(|signal| signal.magnitude).collect();
        program.use_uniform_array("unif_k", &unif_k[..])?;

        let unif_s: Vec<_> = self.signals.iter().map(|signal| signal.scale).collect();
        program.use_uniform_array("unif_s", &unif_s[..])?;

        let unif_b: Vec<_> = self.signals.iter().map(|signal| signal.center).collect();
        program.use_uniform_array("unif_b", &unif_b[..])?;

        let unif_lower = self.range.start;
        program.use_uniform_float("unif_lower", &unif_lower)?;

        let unif_upper = self.range.end;
        program.use_uniform_float("unif_upper", &unif_upper)?;

        Ok(())
    }

    fn render_states(&self) -> RenderStates {
        RenderStates {
            // depth_test: DepthTest::Never,
            // write_mask: WriteMask::COLOR,
            // clip: Clip::Disabled,
            // cull: Cull::None,
            // blend: Blend::Disabled,
            ..Default::default()
        }
    }

    fn is_transparent(&self) -> bool {
        false
    }
}

struct Signal {
    magnitude: f32,
    scale: Vec3,
    center: Vec3,
}

impl<'de> Deserialize<'de> for Signal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct S {
            magnitude: f32,
            scale: [f32; 3],
            center: [f32; 3],
        }
        let S {
            magnitude,
            scale: [scale_x, scale_y, scale_z],
            center: [center_x, center_y, center_z],
        } = S::deserialize(deserializer)?;
        Ok(Self {
            magnitude,
            scale: vec3(scale_x, scale_y, scale_z),
            center: vec3(center_x, center_y, center_z),
        })
    }
}
