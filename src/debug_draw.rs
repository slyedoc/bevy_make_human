use bevy::{animation::AnimationTargetId, color::palettes::css, prelude::*};
use bevy_mod_billboard::prelude::*;

/// Gizmo config for skeleton bone visualization
#[derive(Reflect, GizmoConfigGroup)]
pub struct SkeletonGizmos {
    /// Radius of joint spheres
    pub joint_radius: f32,
    /// Color for root bones
    pub root_color: Color,
    /// Color for child bones
    pub bone_color: Color,
}

impl Default for SkeletonGizmos {
    fn default() -> Self {
        Self {
            joint_radius: 0.005,
            root_color: Color::srgb(1.0, 0.0, 0.0), // Red
            bone_color: Color::srgb(0.0, 1.0, 0.0), // Green
        }
    }
}

/// Gizmo config for joint axes (RGB = XYZ)
#[derive(Reflect, GizmoConfigGroup)]
pub struct JointAxesGizmos {
    /// Length of axis lines
    pub axis_length: f32,
    /// X axis color
    pub x_color: Color,
    /// Y axis color
    pub y_color: Color,
    /// Z axis color
    pub z_color: Color,
    /// Text label scale
    pub label_scale: f32,
    /// Label Y offset above joint
    pub label_offset: f32,
}

impl Default for JointAxesGizmos {
    fn default() -> Self {
        Self {
            axis_length: 0.04,
            x_color: css::RED.into(),
            y_color: css::GREEN.into(),
            z_color: css::BLUE.into(),
            label_scale: 0.001,
            label_offset: 0.03,
        }
    }
}

/// Marker for joint name label entities
#[derive(Component)]
pub struct JointNameLabel;

/// Marker for joints that have labels attached
#[derive(Component)]
struct HasJointLabel;

pub struct MakeHumanDebugPlugin;

impl Plugin for MakeHumanDebugPlugin {
    fn build(&self, app: &mut App) {
        // Skeleton bones gizmo
        app.insert_gizmo_config(
            SkeletonGizmos::default(),
            GizmoConfig {
                enabled: false,   // Off by default
                depth_bias: -1.0, // Render through geometry
                ..Default::default()
            },
        );

        // Joint axes gizmo
        app.insert_gizmo_config(
            JointAxesGizmos::default(),
            GizmoConfig {
                enabled: false, // Off by default
                depth_bias: -1.0,
                ..Default::default()
            },
        );

        app.add_systems(
            Update,
            (
                draw_skeleton_gizmos.run_if(|store: Res<GizmoConfigStore>| {
                    store.config::<SkeletonGizmos>().0.enabled
                }),
                draw_joint_axes_gizmos.run_if(|store: Res<GizmoConfigStore>| {
                    store.config::<JointAxesGizmos>().0.enabled
                }),
                manage_joint_labels,
            ),
        );
    }

    fn finish(&self, app: &mut App) {
        if !app.is_plugin_added::<BillboardPlugin>() {
            app.add_plugins(BillboardPlugin);
        }
    }
}

/// Draw skeleton bones as gizmos for debugging
/// Reads from actual bone entity GlobalTransforms (animated)
pub fn draw_skeleton_gizmos(
    bones: Query<(&GlobalTransform, Option<&ChildOf>), With<AnimationTargetId>>,
    parent_transforms: Query<&GlobalTransform>,
    mut gizmos: Gizmos<SkeletonGizmos>,
    store: Res<GizmoConfigStore>,
    mut logged: Local<bool>,
) {
    let (_, config) = store.config::<SkeletonGizmos>();

    for (global_transform, parent) in bones.iter() {
        let head_world = global_transform.translation();

        // Color based on whether bone has parent
        let color = if parent.is_none() {
            config.root_color
        } else {
            config.bone_color
        };

        // Draw line to parent if exists
        if let Some(child_of) = parent {
            if let Ok(parent_transform) = parent_transforms.get(child_of.parent()) {
                let parent_pos = parent_transform.translation();
                gizmos.line(head_world, parent_pos, color);
            }
        }

        // Draw a small sphere at the joint position
        gizmos.sphere(
            Isometry3d::from_translation(head_world),
            config.joint_radius,
            color,
        );
    }
    *logged = true;
}

/// Draw local XYZ axes at each joint (RGB = XYZ convention)
pub fn draw_joint_axes_gizmos(
    joints: Query<&GlobalTransform, With<AnimationTargetId>>,
    mut gizmos: Gizmos<JointAxesGizmos>,
    store: Res<GizmoConfigStore>,
) {
    let (_, config) = store.config::<JointAxesGizmos>();

    for transform in &joints {
        let pos = transform.translation();
        let rot = transform.to_scale_rotation_translation().1;

        // RGB = XYZ convention (configurable)
        gizmos.line(
            pos,
            pos + rot * Vec3::X * config.axis_length,
            config.x_color,
        );
        gizmos.line(
            pos,
            pos + rot * Vec3::Y * config.axis_length,
            config.y_color,
        );
        gizmos.line(
            pos,
            pos + rot * Vec3::Z * config.axis_length,
            config.z_color,
        );
    }
}

/// Spawn/despawn joint name labels based on JointAxesGizmos enabled state
fn manage_joint_labels(
    mut commands: Commands,
    store: Res<GizmoConfigStore>,
    unlabeled_joints: Query<
        (Entity, Option<&Name>, &GlobalTransform),
        (With<AnimationTargetId>, Without<HasJointLabel>),
    >,
    labeled_joints: Query<Entity, (With<AnimationTargetId>, With<HasJointLabel>)>,
    labels: Query<Entity, With<JointNameLabel>>,
    mut label_transforms: Query<&mut Transform, With<JointNameLabel>>,
) {
    let (gizmo_config, config) = store.config::<JointAxesGizmos>();

    if gizmo_config.enabled {
        // Spawn labels for any joints that don't have them yet (uses default font)
        //let font: Handle<Font> = asset_server.load("fonts/FiraMono-Medium.ttf");
        let font: Handle<Font> = Handle::default();

        for (entity, name, _) in &unlabeled_joints {
            let label = name
                .map(|n| n.to_string())
                .unwrap_or_else(|| "?".to_string());

            commands.entity(entity).insert(HasJointLabel);

            commands
                .spawn((
                    JointNameLabel,
                    ChildOf(entity),
                    BillboardText::default(),
                    TextLayout::new_with_justify(Justify::Left),
                    Transform {
                        translation: Vec3::Y * config.label_offset,
                        scale: Vec3::splat(config.label_scale),
                        ..default()
                    },
                ))
                .with_child((
                    Name::new("DebugLabel"),
                    TextSpan::new(label),
                    TextFont::from(font.clone()).with_font_size(16.0),
                    TextColor(css::WHITE.into()),
                ));
        }

        // Update existing label transforms when config changes
        for mut transform in &mut label_transforms {
            transform.translation.y = config.label_offset;
            transform.scale = Vec3::splat(config.label_scale);
        }
    } else {
        // Despawn all labels and remove markers
        for entity in &labels {
            commands.entity(entity).despawn();
        }
        for entity in &labeled_joints {
            commands.entity(entity).remove::<HasJointLabel>();
        }
    }
}
