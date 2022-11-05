use bevy::{prelude::*, render::camera::RenderTarget};

mod enemy;
mod flow_field;
mod grid;
mod gun;
mod wall;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MouseWorldPos(Vec2::ONE * 10000.0))
            .add_plugin(flow_field::FlowFieldPlugin)
            .add_plugin(grid::GridPlugin)
            .add_plugin(enemy::EnemyPlugin)
            .add_plugin(wall::WallPlugin)
            .add_plugin(gun::GunPlugin)
            .add_system(update_mouse_position);
    }
}

pub struct MouseWorldPos(Vec2);

fn update_mouse_position(
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut mouse_pos: ResMut<MouseWorldPos>,
) {
    //let (camera, camera_transform) = q_camera.single();
    for (camera, camera_transform) in q_camera.iter() {
        let win = if let RenderTarget::Window(id) = camera.target {
            windows.get(id).unwrap()
        } else {
            windows.get_primary().unwrap()
        };

        if let Some(screen_pos) = win.cursor_position() {
            let window_size = Vec2::new(win.width() as f32, win.height() as f32);

            // convert screen position [0..resolution] to ndc [-1..1] (gpu coords)
            let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

            // matrix for undoing the projection and camera transform
            let ndc_to_world =
                camera_transform.compute_matrix() * camera.projection_matrix().inverse();

            // use it to convert ndc to world-space coordinates
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

            // reduce it to a 2D value
            let world_pos: Vec2 = world_pos.truncate();

            mouse_pos.0 = world_pos;
        }
    }
}
