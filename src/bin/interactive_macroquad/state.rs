use crate::model::{AppScene, GameState, LaunchConfig, LaunchEditor, Level, SurfaceEditor};

pub(crate) struct AppRuntime {
    pub(crate) levels: Vec<Level>,
    pub(crate) current_level_idx: usize,
    pub(crate) highest_unlocked_level: usize,
    pub(crate) config: LaunchConfig,
    pub(crate) game: GameState,
    pub(crate) show_preview: bool,
    pub(crate) sim_speed: f32,
    pub(crate) scene: AppScene,
    pub(crate) surface_editor: SurfaceEditor,
    pub(crate) launch_editor: LaunchEditor,
}

impl AppRuntime {
    pub(crate) fn new() -> Self {
        let levels = Level::campaign();
        let current_level_idx = 0usize;
        let config = levels[current_level_idx].default_launch;
        Self {
            levels,
            current_level_idx,
            highest_unlocked_level: 0,
            config,
            game: GameState::new(),
            show_preview: true,
            sim_speed: 1.0,
            scene: AppScene::Title,
            surface_editor: SurfaceEditor::new(),
            launch_editor: LaunchEditor::new(),
        }
    }

    pub(crate) fn current_level(&self) -> &Level {
        &self.levels[self.current_level_idx]
    }

    pub(crate) fn levels_len(&self) -> usize {
        self.levels.len()
    }

    pub(crate) fn load_current_level_defaults(&mut self) {
        self.config = self.current_level().default_launch;
        self.game.reset();
    }

    pub(crate) fn set_loaded_status(&mut self) {
        self.game.status_line = format!("Loaded {}", self.current_level().code);
    }

    pub(crate) fn set_moved_status(&mut self) {
        self.game.status_line = format!("Moved to {}", self.current_level().code);
    }

    pub(crate) fn set_advanced_status(&mut self) {
        self.game.status_line = format!("Advanced to {}", self.current_level().code);
    }
}
