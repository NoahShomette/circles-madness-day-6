use bevy::{prelude::*, time::Stopwatch};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .init_resource::<LastActivity>()
            .add_systems(Startup, (setup_controls_hint, setup_menu))
            .add_systems(OnEnter(GameState::Menu), display_menu)
            .add_systems(OnExit(GameState::Menu), hide_menu)
            .add_systems(
                Update,
                (
                    display_controls_hint,
                    resume_game.run_if(in_state(GameState::Menu)),
                    pause_game.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, States)]
pub enum GameState {
    Menu,
    #[default]
    Playing,
}

#[derive(Resource, Default)]
pub struct LastActivity(pub Stopwatch);

#[derive(Component)]
struct ControlsHint;

#[derive(Component)]
struct MenuNode;

fn setup_controls_hint(mut commands: Commands) {
    commands.spawn((
        TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "WASD/Arrows: Move\nMouse/Click: Aim and shoot/Enter: Pause\n"
                        .to_string(),
                    style: TextStyle {
                        font_size: 30.,
                        ..default()
                    },
                }],
                ..default()
            },
            style: Style {
                margin: UiRect {
                    left: Val::Percent(2.),
                    bottom: Val::Percent(2.),
                    ..default()
                },
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        },
        ControlsHint,
    ));
}

fn setup_menu(mut commands: Commands) {
    let menu_node = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_content: AlignContent::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::GRAY.with_a(0.5)),
                ..default()
            },
            MenuNode,
        ))
        .id();

    let text_node = commands
        .spawn((TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "Press ENTER to start/resume the game\n".to_string(),
                    style: TextStyle {
                        font_size: 50.,
                        ..default()
                    },
                }],
                ..default()
            },
            style: Style { ..default() },
            ..default()
        },))
        .id();

    commands.entity(menu_node).add_child(text_node);
}

fn display_controls_hint(
    mut controls_hint: Query<&mut Style, With<ControlsHint>>,
    mut last_activity: ResMut<LastActivity>,
    time: Res<Time>,
    game_state: Res<State<GameState>>,
) {
    let mut controls_hint = controls_hint.single_mut();
    if game_state.get() == &GameState::Menu {
        controls_hint.display = Display::DEFAULT;
    } else {
        last_activity.0.tick(time.delta());
        if last_activity.0.elapsed_secs() > 1_f32 {
            controls_hint.display = Display::DEFAULT;
        } else {
            controls_hint.display = Display::None;
        }
    }
}

fn display_menu(mut menu: Query<&mut Style, With<MenuNode>>) {
    menu.single_mut().display = Display::DEFAULT;
}

fn hide_menu(mut menu: Query<&mut Style, With<MenuNode>>) {
    menu.single_mut().display = Display::None;
}

fn resume_game(
    keyboard_input: Res<Input<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut last_activity: ResMut<LastActivity>,
) {
    if keyboard_input.just_pressed(KeyCode::Return) {
        game_state.0 = Some(GameState::Playing);
        last_activity.0.reset();
    }
}

fn pause_game(keyboard_input: Res<Input<KeyCode>>, mut game_state: ResMut<NextState<GameState>>) {
    if keyboard_input.just_pressed(KeyCode::Return) {
        game_state.0 = Some(GameState::Menu);
    }
}
