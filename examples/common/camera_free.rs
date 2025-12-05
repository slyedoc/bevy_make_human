use bevy::{
    ecs::spawn::SpawnWith,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_enhanced_input::prelude::*;

pub struct CameraFreePlugin;

impl Plugin for CameraFreePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EnhancedInputPlugin>() {
            app.add_plugins(EnhancedInputPlugin);
        }
        app.add_input_context::<CameraFree>()
            .add_observer(on_insert_camera)
            .add_observer(apply_movement)
            .add_observer(apply_assend)
            .add_observer(apply_rotate)
            .add_observer(on_right_mouse_hold)
            .add_observer(on_right_mouse_release);
    }
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[require(Name::new("CameraFree"))]
pub struct CameraFree {
    pub normal_speed: f32,
    pub sprint_speed: f32,
    pub crawl_speed: f32,
}

impl Default for CameraFree {
    fn default() -> Self {
        Self {
            normal_speed: 0.1,
            sprint_speed: 0.2,
            crawl_speed: 0.01,
        }
    }
}

impl CameraFree {
    pub fn new(base: f32) -> Self {
        Self {
            normal_speed: base,
            sprint_speed: base * 2.0,
            crawl_speed: base * 0.5,
        }
    }
}

fn on_insert_camera(
    trigger: On<Insert, CameraFree>,
    query: Query<&CameraFree>,
    mut commands: Commands,
) {
    let CameraFree { normal_speed, sprint_speed, crawl_speed } = *query.get(trigger.entity).unwrap();

    commands
        .entity(trigger.entity)
        .insert(Actions::<CameraFree>::spawn(SpawnWith(
            move |context: &mut ActionSpawner<_>| {
                // Create the right mouse button hold action
                let right_mouse = context
                    .spawn((
                        Action::<RightMouseHold>::new(),
                        bindings![MouseButton::Right],
                    ))
                    .id();

                // Create modifier key actions for Chord conditions
                let ctrl = context
                    .spawn((
                        Action::<SlowMode>::new(),
                        bindings![KeyCode::ControlLeft, KeyCode::ControlRight],
                    ))
                    .id();

                let shift = context
                    .spawn((
                        Action::<SprintMode>::new(),
                        bindings![KeyCode::ShiftLeft, KeyCode::ShiftRight],
                    ))
                    .id();

                // Create the rotate action with mouse motion that only works when right mouse is held
                context.spawn((
                    Action::<Rotate>::new(),
                    Chord::single(right_mouse), // Only fire when right mouse is held
                    Bindings::spawn((
                        Spawn((Binding::mouse_motion(), Scale::splat(0.1), Negate::all())),
                        Axial::right_stick(),
                    )),
                ));

                // Sprint movement (Shift + WASD) - 2x speed
                context.spawn((
                    Action::<Move>::new(),
                    Chord::single(shift),
                    DeadZone::default(),
                    SmoothNudge::default(),
                    Scale::splat(sprint_speed),
                    Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
                ));

                // Slow movement (Ctrl + WASD) - 20% speed
                context.spawn((
                    Action::<Move>::new(),
                    Chord::single(ctrl),
                    DeadZone::default(),
                    SmoothNudge::default(),
                    Scale::splat(crawl_speed),
                    Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
                ));

                // Normal movement (WASD) - base speed
                context.spawn((
                    Action::<Move>::new(),
                    DeadZone::default(),
                    SmoothNudge::default(),
                    Scale::splat(normal_speed),
                    Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
                ));

                // Sprint ascend (Shift + Q/E)
                context.spawn((
                    Action::<Assend>::new(),
                    Chord::single(shift),
                    SmoothNudge::default(),
                    Scale::splat(sprint_speed),
                    Bindings::spawn((Bidirectional {
                        positive: Binding::from(KeyCode::KeyQ),
                        negative: Binding::from(KeyCode::KeyE),
                    },)),
                ));

                // Slow ascend (Ctrl + Q/E)
                context.spawn((
                    Action::<Assend>::new(),
                    Chord::single(ctrl),
                    SmoothNudge::default(),
                    Scale::splat(crawl_speed),
                    Bindings::spawn((Bidirectional {
                        positive: Binding::from(KeyCode::KeyQ),
                        negative: Binding::from(KeyCode::KeyE),
                    },)),
                ));

                // Normal ascend/descend (Q/E) - base speed
                context.spawn((
                    Action::<Assend>::new(),
                    SmoothNudge::default(),
                    Scale::splat(normal_speed),
                    Bindings::spawn((Bidirectional {
                        positive: Binding::from(KeyCode::KeyQ),
                        negative: Binding::from(KeyCode::KeyE),
                    },)),
                ));
            },
        )));
}

fn on_right_mouse_hold(
    _trigger: On<Start<RightMouseHold>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    // Grab cursor when right mouse is pressed
    cursor_options.grab_mode = CursorGrabMode::Confined;
    cursor_options.visible = false;
}

fn on_right_mouse_release(
    _trigger: On<Complete<RightMouseHold>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    // Release cursor when right mouse is released
    cursor_options.grab_mode = CursorGrabMode::None;
    cursor_options.visible = true;
}

fn apply_movement(trigger: On<Fire<Move>>, mut query: Query<&mut Transform>) {
    if let Ok(mut trans) = query.get_mut(trigger.context) {
        // Move to the camera direction.
        let rotation = trans.rotation;

        // Movement consists of X and -Z components, so swap Y and Z with negation.
        // We could do it with modifiers, but it would be weird for an action to return
        // a `Vec3` like this, so we doing it inside the function.
        let mut movement = trigger.value.extend(0.0).xzy();
        movement.z = -movement.z;
        trans.translation += rotation * movement;
    }
}

fn apply_assend(trigger: On<Fire<Assend>>, mut query: Query<&mut Transform, With<Camera>>) {
    if let Ok(mut trans) = query.get_mut(trigger.context) {
        let change = vec3(0.0, trigger.value, 0.0);
        let rot = trans.rotation;
        trans.translation += rot * change;
    }
}

fn apply_rotate(trigger: On<Fire<Rotate>>, mut query: Query<&mut Transform>) {
    let mut transform = query.get_mut(trigger.context).unwrap();
    let (mut yaw, mut pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);

    yaw += trigger.value.x.to_radians();
    pitch += trigger.value.y.to_radians();

    pitch = pitch.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
}

#[derive(InputAction)]
#[action_output(Vec2)]
struct Move;

#[derive(InputAction)]
#[action_output(f32)]
struct Assend;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Rotate;

#[derive(InputAction)]
#[action_output(bool)]
struct RightMouseHold;

#[derive(InputAction)]
#[action_output(bool)]
struct SlowMode;

#[derive(InputAction)]
#[action_output(bool)]
struct SprintMode;
