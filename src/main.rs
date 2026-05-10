use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::{prelude::*, prelude::SliceRandom, rngs::StdRng, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};
use std::f32::consts::PI;

const CELL_SPACING: f32 = 192.5;


#[derive(Component)]
pub struct QuizChoice {
    pub is_correct: bool,
    pub selected: bool,
}

#[derive(Component)]
pub struct SelectionHighlight;

#[derive(Component)]
pub struct SubmitButton;

#[derive(Component)]
pub struct FeedbackIndicator;

#[derive(Component, Reflect, Clone, Default, PartialEq)]
#[reflect(Component)]
pub struct MatrixCell {
    pub row: u32,
    pub col: u32,
    pub layers: Vec<CellLayer>,
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct CellLayer {
    pub shape: ShapeType,
    pub scale: f32,
    pub rotation: f32,
    pub offset: Vec2,
    pub fill: FillStyle,
    pub count: u8,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq)]
pub enum FillStyle {
    Solid,
    Outline(f32),
}

#[derive(Reflect, Clone, Copy, Default, PartialEq, Debug)]
pub enum ShapeType {
    #[default]
    Square,
    Circle,
    Triangle,
    Diamond,
    Hexagon,
    ThickLine,
    Line,
    Pentagon,
    Ellipse
}

#[derive(Debug, Clone, Copy)]
pub enum MatrixRule {
    Constant { layer_idx: usize },
    IncrementalRotation { layer_idx: usize, step: usize },
    ShapeShift { layer_idx: usize, step: usize },
    NumericOrdering { layer_idx: usize, step: usize },
    AdditiveLogic { layer_idx: usize },
}

impl MatrixRule {
    pub fn apply(&self, grid: &mut [Option<MatrixCell>; 9], rng: &mut StdRng) {
        match self {
            Self::Constant { layer_idx: _ } => {}
            Self::IncrementalRotation { layer_idx, step } => {
                    let rotation_step = PI / 3.0;
                    let rotation = (*step as f32 + 1.0) * rotation_step;
                for r in 0..3 {
                    for c in 0..3 {
                        if let Some(cell) = grid[r * 3 + c].as_mut() {
                            if let Some(layer) = cell.layers.get_mut(*layer_idx) {
                                layer.rotation = ((r +1) * (c + 1)) as f32 * rotation;
                            }
                        }
                    }
                }
            }
            Self::ShapeShift { layer_idx, step } => {
                let mut row_shapes = vec![
                    ShapeType::ThickLine,
                    ShapeType::Ellipse,
                    ShapeType::Triangle,
                    ShapeType::Pentagon,
                    ShapeType::Hexagon,
                ];
                let mut nums: usize = *(3..row_shapes.len()).collect::<Vec<usize>>().choose(rng).unwrap();
                for _ in 0..=nums {
                    row_shapes.shuffle(rng);
                }


                let row_shapes_modu = row_shapes.len();
                for _ in 0..=*layer_idx {
                    row_shapes.shuffle(rng);
                }
                
                let grid_len = grid.len();
                for index in 0..grid_len {
                    let layer = grid[index].as_mut().unwrap().layers.get_mut(*layer_idx).unwrap();
                    layer.shape = row_shapes[( index + *step * index) % row_shapes_modu];
                }
            }
            MatrixRule::NumericOrdering { layer_idx, step } => {
                let grid_len = grid.len();
                let idx_offset = *(1..=5).collect::<Vec<usize>>().choose(rng).unwrap(); 
                for index in 0..grid_len {
                    let layer = grid[index].as_mut().unwrap().layers.get_mut(*layer_idx).unwrap();
                    layer.count = ((( *step * index) % 5) + idx_offset)  as u8 ;
                }
            }
            _ => {}
        }
    }
}

fn spawn_daily_puzzle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let days_since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() / 86_400;
    let seed = days_since_epoch;
    let mut rng = StdRng::seed_from_u64(seed);

    let mut cells: [Option<MatrixCell>; 9] = Default::default();

    let all_shapes = vec![
        ShapeType::ThickLine,
        ShapeType::Ellipse,
        ShapeType::Triangle,
        ShapeType::Square,
        ShapeType::Pentagon,
        ShapeType::Hexagon,
    ];

    let layer1_shape = all_shapes[0];
    let layer2_shape = all_shapes[1];
    let layer3_shape = all_shapes[2];

    let cell_layer1 = CellLayer {
        shape: layer1_shape,
        scale: 125.0,
        rotation: 0.0,
        offset: Vec2::new(0.0, 0.0),
        fill: FillStyle::Outline(0.2),
        count: 1,
    };

    let cell_layer2 = CellLayer {
        shape: layer2_shape,
        scale: 62.5,
        rotation: 0.0,
        offset: Vec2::new(0.0, 0.0),
        fill: FillStyle::Outline(0.2),
        count: 1,
    };

    let cell_layer3 = CellLayer {
        shape: layer3_shape,
        scale: 31.25,
        rotation: 0.0,
        offset: Vec2::new(0.0, 0.0),
        fill: FillStyle::Outline(0.2),
        count: 1,
    };

    for i in 0..9 {
        cells[i] = Some(MatrixCell {
            row: (i / 3) as u32,
            col: (i % 3) as u32,
            layers: vec![cell_layer1.clone(), cell_layer2.clone(), cell_layer3.clone()],
        });
    }

    let rules: Vec<MatrixRule> = make_rule_for_puzzle(seed, 2);

    for rule in &rules {
        rule.apply(&mut cells, &mut rng);
    }

    // --- GENERATE MULTIPLE CHOICE OPTIONS ---
    let correct_answer = cells[8].clone().unwrap();
    let mut quiz_options = vec![correct_answer.clone()];

    while quiz_options.len() < 6 {
        let mut distractor = correct_answer.clone();
        let num_mutations = rng.random_range(1..=2);

        for _ in 0..num_mutations {
            let layer_idx = rng.random_range(0..distractor.layers.len());
            match rng.random_range(0..3) {
                0 => {
                    distractor.layers[layer_idx].shape = *all_shapes.choose(&mut rng).unwrap();
                }
                1 => {
                    distractor.layers[layer_idx].count = rng.random_range(1..=4);
                }
                _ => {
                    distractor.layers[layer_idx].rotation += PI / 4.0;
                }
            }
        }

        let is_duplicate = quiz_options.iter().any(|opt| {
            opt.layers.iter().zip(distractor.layers.iter()).all(|(a, b)| {
                a.shape == b.shape && a.count == b.count
            })
        });

        if !is_duplicate {
            quiz_options.push(distractor);
        }
    }

    quiz_options.shuffle(&mut rng);

    // --- SPAWN MAIN 3x3 MATRIX ---
    let grid_x_offset = 0.0;
    let grid_y_offset = 100.0;
    let mut target_cell_pos = Vec2::ZERO;

    for (i, celli) in cells.into_iter().enumerate() {
        let cell = celli.unwrap();
        let x = (cell.col as f32 - 1.0) * (CELL_SPACING + 200.0) - grid_x_offset;
        let y = (1.0 - cell.row as f32) * (CELL_SPACING - 20.0) + grid_y_offset;

        if i == 8 {
            target_cell_pos = Vec2::new(x, y);
        }

        commands
            .spawn((
                cell.clone(),
                Transform::from_xyz(x, y, 0.0),
                Visibility::Visible,
                InheritedVisibility::default(),
            ))
            .with_children(|parent| {
                if i != 8 {
                    for (z, layer) in cell.layers.iter().enumerate() {
                        let color = match z {
                            0 => Color::BLACK,
                            1 => Color::WHITE,
                            2 => bevy::prelude::Color::Hsla(Hsla::new(120.0, 1.0, 0.25, 1.0)),
                            _ => bevy::prelude::Color::Hsla(Hsla::new(0.2, 0.99, 0.75, 0.0)),
                        };

                        for a in 0..layer.count {
                            let relative_coordinate =
                                index_to_coordinates(a, layer.count, layer.count % 2 == 1);
                            let scalar = 12.5 * 3.0 / (2.0 * z as f32 + 1.0);
                            let rhs =  relative_coordinate * scalar;

                            parent.spawn((
                                Mesh2d(get_mesh_for_shape(layer.shape, &mut meshes)),
                                MeshMaterial2d(materials.add(color)),
                                Transform {
                                    translation: rhs.extend(z as f32 * 0.0),
                                    rotation: Quat::from_rotation_z(layer.rotation),
                                    scale: Vec3::splat(layer.scale),
                                },
                            ));
                        }
                    }
                } else {
                    // Blank placeholder box for the target missing piece
                    parent.spawn((
                        Mesh2d(meshes.add(Rectangle::new(CELL_SPACING * 0.8, CELL_SPACING * 0.8))),
                        MeshMaterial2d(
                            materials.add(bevy::prelude::Color::Hsla(Hsla::new(0.0, 0.0, 0.5, 0.15))),
                        ),
                        Transform::default(),
                    ));
                }
            });
    }

    // --- SPAWN FEEDBACK CHECKMARK (HIDDEN INITIALLY) ---
    // Placed directly over the target missing piece location
    commands
        .spawn((
            FeedbackIndicator,
            Transform::from_xyz(target_cell_pos.x, target_cell_pos.y, 10.0),
            Visibility::Hidden,
            InheritedVisibility::default(),
        ))
        .with_children(|parent| {
            let green_mat = materials.add(Color::srgb(0.1, 0.85, 0.2));
            
            // Short left leg
            parent.spawn((
                Mesh2d(meshes.add(Rectangle::new(12.0, 35.0))),
                MeshMaterial2d(green_mat.clone()),
                Transform::from_xyz(-15.0, -5.0, 0.0)
                    .with_rotation(Quat::from_rotation_z(-PI / 4.0)),
            ));
            // Long right leg
            parent.spawn((
                Mesh2d(meshes.add(Rectangle::new(12.0, 70.0))),
                MeshMaterial2d(green_mat),
                Transform::from_xyz(10.0, 10.0, 0.0)
                    .with_rotation(Quat::from_rotation_z(PI / 4.0)),
            ));
        });

    // --- SPAWN MULTIPLE CHOICE ROW ---
    let choice_spacing = 200.0;
    let choices_y = -260.0;

    for (idx, option_cell) in quiz_options.iter().enumerate() {
        let x = (idx as f32 - 2.5) * choice_spacing;
        let is_correct_choice = *option_cell == correct_answer;

        commands
            .spawn((
                option_cell.clone(),
                QuizChoice {
                    is_correct: is_correct_choice,
                    selected: false,
                },
                Transform::from_xyz(x, choices_y, 0.0).with_scale(Vec3::splat(0.65)),
                Visibility::Visible,
                InheritedVisibility::default(),
            ))
            .with_children(|parent| {
                // Selection Highlight Background Panel (Hidden initially)
                let parent_id = parent.spawn((
                    SelectionHighlight,
                    Mesh2d(meshes.add(Rectangle::new(190.0, 190.0))),
                    MeshMaterial2d(materials.add(Color::Hsla(Hsla::new(0.58, 0.9, 0.6, 0.4)))),
                    Transform::from_xyz(0.0, 0.0, -1.0),
                    Visibility::Hidden,
                ));

                for (z, layer) in option_cell.layers.iter().enumerate() {
                    let color = match z {
                        0 => Color::BLACK,
                        1 => Color::WHITE,
                        2 => bevy::prelude::Color::Hsla(Hsla::new(120.0, 1.0, 0.25, 1.0)),
                        _ => bevy::prelude::Color::Hsla(Hsla::new(0.2, 0.99, 0.75, 0.0)),
                    };

                    for a in 0..layer.count {
                        let relative_coordinate =
                            index_to_coordinates(a, layer.count, layer.count % 2 == 1);
                        let scalar = 12.5 * 3.0 / (2.0 * z as f32 + 1.0);
                        let rhs = relative_coordinate * scalar;

                        parent.spawn((
                            Mesh2d(get_mesh_for_shape(layer.shape, &mut meshes)),
                            MeshMaterial2d(materials.add(color)),
                            Transform {
                                translation: rhs.extend(z as f32 * 0.0),
                                rotation: Quat::from_rotation_z(0.0),
                                scale: Vec3::splat(layer.scale),
                            },
                        ));
                    }
                }
            });
    }

    // --- SPAWN SUBMIT BUTTON ---
    let submit_y = -370.0;
    commands
        .spawn((
            SubmitButton,
            Mesh2d(meshes.add(Rectangle::new(200.0, 50.0))),
            MeshMaterial2d(materials.add(Color::srgb(0.2, 0.5, 0.8))),
            Transform::from_xyz(0.0, submit_y, 0.0),
            Visibility::Visible,
            InheritedVisibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2d::new("SUBMIT"),
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 1.0),
            ));
        });
}

// --- INTERACTION SYSTEM ---
fn handle_quiz_interaction(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_choices: Query<(Entity, &mut QuizChoice, &GlobalTransform, &Children)>,
    mut q_highlights: Query<&mut Visibility, With<SelectionHighlight>>,
    q_submit: Query<&GlobalTransform, With<SubmitButton>>,
    mut q_checkmark: Query<&mut Visibility, (With<FeedbackIndicator>, Without<SelectionHighlight>)>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = q_windows.single() else { return };
    let Ok((camera, camera_transform)) = q_camera.single() else { return };

    // Get cursor position in 2D world space
    let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    else {
        return;
    };

    // 1. Check if an option was clicked
    let mut clicked_entity = None;
    for (entity, _, transform, _) in q_choices.iter() {
        let pos = transform.translation().truncate();
        // Option bounding radius (~75 units scaled)
        if world_position.distance(pos) < 75.0 {
            clicked_entity = Some(entity);
            break;
        }
    }

    // Apply exclusive selection
    if let Some(target_ent) = clicked_entity {
        for (entity, mut choice, _, children) in q_choices.iter_mut() {
            let is_selected = entity == target_ent;
            choice.selected = is_selected;

            // Toggle highlight background visibility
            for child in children.iter() {
                if let Ok(mut vis) = q_highlights.get_mut(child) {
                    *vis = if is_selected {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
            }
        }
        return; // Clicked a choice, no need to check submit button
    }

    // 2. Check if the Submit button was clicked
    if let Ok(submit_transform) = q_submit.single() {
        let submit_pos = submit_transform.translation().truncate();
        let diff = (world_position - submit_pos).abs();
        
        // Button bounds check (200x50 -> half extents 100x25)
        if diff.x < 100.0 && diff.y < 25.0 {
            let mut is_answer_correct = false;
            for (_, choice, _, _) in q_choices.iter() {
                if choice.selected && choice.is_correct {
                    is_answer_correct = true;
                    break;
                }
            }

            // Reveal green checkmark if correct, hide if incorrect
            for mut vis in q_checkmark.iter_mut() {
                *vis = if is_answer_correct {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(bevy::prelude::Color::Hsla(Hsla::new(240.0, 0.99, 0.55, 1.0))))
        .register_type::<MatrixCell>()
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, spawn_daily_puzzle)
        .add_systems(Update, handle_quiz_interaction) // Registered input listener
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn get_mesh_for_shape(shape: ShapeType, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    match shape {
        ShapeType::Square => meshes.add(Rectangle::new(0.5,0.5)),
        ShapeType::Circle => meshes.add(Circle::new(0.5)),
        ShapeType::Triangle => meshes.add(Triangle2d::new(vec2(0.5,0.0), vec2(-0.5,0.0), vec2(0.0,0.5))),
        ShapeType::Diamond => meshes.add(RegularPolygon::new(0.5, 4)),
        ShapeType::ThickLine => meshes.add(Rectangle::new(0.2, 0.5)),
        ShapeType::Line => meshes.add(Rectangle::new(0.1, 0.5)),
        ShapeType::Hexagon => meshes.add(RegularPolygon::new(0.5, 6)),
        ShapeType::Pentagon => meshes.add(RegularPolygon::new(0.5, 5)),
        ShapeType::Ellipse => meshes.add(Ellipse::new(0.5, 0.25))
    }
}

fn index_to_coordinates(index: u8, width: u8, odd_len: bool) -> Vec2 {
    let mut x = (index % width) as f32;
    let y = index as f32 / width as f32;

    if !odd_len {
        x += 0.5;
    }

    let mid_x = (width / 2) as f32;
    let mid_y = (width / 2) as f32;

    let x_rel = x - mid_x;
    let y_rel = y - mid_y;

    if x_rel == 0.0 && y_rel == 0.0 {
        return vec2(0.0, 0.0);
    }

    if x_rel == 0.0 {
        vec2(0.0, 0.5)
    } else {
        vec2(x_rel, -0.5)
    }
}


fn make_rule_for_puzzle(seed: u64, difficulty: usize, ) -> Vec<MatrixRule> {
    let mut rng = StdRng::seed_from_u64(seed);
    let rand_range = (0..120).collect::<Vec<usize>>();
    let mut randomizer = rand_range.choose(&mut rng).clone().unwrap();
    let step_range_mapped = match difficulty {
        0..=4 => 0,
        7..=8 => 1,
        9 => 2,
        _ => 3
    };
    let mut final_ruleset: Vec<MatrixRule> = vec![];

        for i in 0..=2 {
            let layer_idx = (randomizer + i)  % 3;
            for j in 0..=2 {
                
                let rule_idx = (randomizer + j)  % 3;
                let step = step_range_mapped;
                    let rule = match rule_idx {
                        0 => MatrixRule::ShapeShift {layer_idx , step },
                        1 => MatrixRule::NumericOrdering { layer_idx, step },
                        2 => MatrixRule::IncrementalRotation { layer_idx, step:   step },
                        _ => MatrixRule::IncrementalRotation { layer_idx, step:   step },
                    };
                    final_ruleset.push(rule);
            }
            
        }
        final_ruleset.shuffle(&mut rng);
        final_ruleset = final_ruleset[0..difficulty].to_vec();
        for r in final_ruleset.clone() {
            println!("{:?}", r);
        }
        final_ruleset
    }
        
