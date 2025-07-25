use std::fs::File;

use egui::{Color32, LayerId, Pos2, Rect, Vec2};
use egui_keybind::Bind;
use egui_overlay::EguiOverlay;
use egui_render_three_d::{ThreeDBackend, three_d::context};

use crate::{
    app::App,
    unreal::{
        global::STRING_CACHE,
        offsets::{
            LOCALPLAYERS, OWNING_GAME_INSTANCE, PLAYER_CAMERA_MANAGER, PLAYER_CHARACTER,
            PLAYER_CONTROLLER_OFFSET,
        },
        types::structs::{FName, TArray, UObject},
    },
};

impl EguiOverlay for App {
    fn gui_run(
        &mut self,
        egui_context: &egui::Context,
        _three_d_backend: &mut ThreeDBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {
        let start = std::time::Instant::now();

        if !self.init {
            self.init = true;

            // Set glfw_backend window position to (0, 0) and normal size
            glfw_backend.window_size_virtual = [1920u32, 1080u32];
            glfw_backend.window.set_size(1920, 1080);
            glfw_backend.window.set_pos(0, 0);

            // Remove shadows
            let mut visuals = egui::Visuals::default();

            visuals.window_shadow = egui::epaint::Shadow {
                offset: Vec2::default(),
                blur: 0.0,
                spread: 0.0,
                color: Color32::from_rgb(0, 0, 0),
            };

            visuals.popup_shadow = egui::epaint::Shadow {
                offset: Vec2::default(),
                blur: 0.0,
                spread: 0.0,
                color: Color32::from_rgb(0, 0, 0),
            };

            egui_context.set_visuals(visuals);
        }

        if self.visible {
            self.draw_main_ui(egui_context);
        }

        // Set Addresses

        let gamestate: usize = self.process.read(self.uworld + 0x2c8).unwrap();
        self.gamestate = gamestate;

        let persistant_level: usize = self.process.read(self.uworld + 0x38).unwrap();
        if persistant_level == 0 {
            return;
        }
        let actors_array: TArray<usize> = self.process.read(persistant_level + 0xA0).unwrap();
        if actors_array.is_empty() {
            return;
        }
        let actors = actors_array.read_all::<usize>().unwrap();
        if actors.is_empty() {
            return;
        }

        self.actors = actors;

        let game_instance_address = match self
            .process
            .read::<usize>(self.uworld + OWNING_GAME_INSTANCE)
        {
            Ok(addr) => addr,
            Err(_) => return,
        };

        let localplayers_address = match self
            .process
            .read::<usize>(game_instance_address + LOCALPLAYERS)
        {
            Ok(addr) => addr,
            Err(_) => return,
        };

        let localplayer_address: usize = match self.process.read(localplayers_address) {
            Ok(addr) => addr,
            Err(_) => return,
        };

        let player_controller_address = match self
            .process
            .read::<usize>(localplayer_address + PLAYER_CONTROLLER_OFFSET)
        {
            Ok(addr) => addr,
            Err(_) => return,
        };

        let player_camera_manager = match self
            .process
            .read::<usize>(player_controller_address + PLAYER_CAMERA_MANAGER)
        {
            Ok(addr) => addr,
            Err(_) => return,
        };

        self.playercam = player_camera_manager;

        self.change_weapon_damage();

        // Create Debug printer
        if self.esp_enabled {
            let painter = egui::Painter::new(
                egui_context.clone(),
                LayerId::debug(),
                Rect {
                    min: Pos2 { x: 0.0, y: 0.0 },
                    max: Pos2 {
                        x: 1920.0,
                        y: 1080.0,
                    },
                },
            );

            self.draw_player_esp(painter.clone());
            self.draw_esp_test(painter.clone());
        }
        // Input Handling
        if egui_context.wants_pointer_input() || egui_context.wants_keyboard_input() {
            glfw_backend.set_passthrough(false);
        } else {
            glfw_backend.set_passthrough(true)
        }
        if egui_context.input_mut(|i| self.open_key.pressed(i)) {
            self.visible = !self.visible;
        }

        let end = std::time::Instant::now();
        let duration = end.duration_since(start);
        println!("Drawing UI took: {:?}", duration);
        self.frames = duration.as_millis() as f32;
        egui_context.request_repaint();
    }
}
