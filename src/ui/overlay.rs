use std::fs::File;

use egui::{Color32, LayerId, Pos2, Rect, Vec2};
use egui_overlay::EguiOverlay;
use egui_render_three_d::ThreeDBackend;

use crate::{
    app::App,
    unreal::{
        global::STRING_CACHE,
        offsets::{
            LOCALPLAYERS, OWNING_GAME_INSTANCE, PLAYER_CAMERA_MANAGER, PLAYER_CONTROLLER_OFFSET,
        },
        types::structs::{FName, TArray, UObject},
    },
};

impl EguiOverlay for App {
    #[flamer::flame]
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

        // Set Addresses
        let gamestate: usize = self.process.read(self.uworld + 0x2c8).unwrap();
        self.gamestate = gamestate;

        let persistant_level: usize = self.process.read(self.uworld + 0x38).unwrap();
        let actors_array: TArray<usize> = self.process.read(persistant_level + 0xA0).unwrap();
        let actors = actors_array.read_all::<usize>().unwrap();

        self.actors = actors;

        //let ulevels: TArray<usize> = self.process.read(self.uworld + 0x2e8).unwrap();
        //let levels = ulevels.read_all::<usize>().unwrap();

        //let mut actors_array: Vec<usize> = Vec::new();
        //for level in levels {
        //    let actor_cluster: usize = self.process.read(level + 0xe0).unwrap();
        //    println!("actor_cluster {:X}", actor_cluster);
        //    if actor_cluster == 0 {
        //        continue;
        //    }
        //    let actors: TArray<usize> = self.process.read(actor_cluster + 0x30).unwrap();
        //    let all_actors = actors.read_all::<usize>().unwrap();
        //    // push all actors to actors_array
        //    actors_array.extend(all_actors);
        //    println!("{:?}", actors);
        //}
        //
        //self.actors = actors_array;

        let game_instance_address = self
            .process
            .read::<usize>(self.uworld + OWNING_GAME_INSTANCE)
            .expect("IDK");

        let localplayers_address = self
            .process
            .read::<usize>(game_instance_address + LOCALPLAYERS)
            .unwrap();
        let localplayer_address: usize = self.process.read(localplayers_address).unwrap();
        let player_controller_address = self
            .process
            .read::<usize>(localplayer_address + PLAYER_CONTROLLER_OFFSET)
            .unwrap();
        let player_camera_manager = self
            .process
            .read::<usize>(player_controller_address + PLAYER_CAMERA_MANAGER)
            .unwrap();

        let player_pawn: UObject = self
            .process
            .read(player_controller_address + 0x3f8)
            .unwrap();

        println!("Localplayer Name {}", player_pawn.index);

        self.playercam = player_camera_manager;

        // Actual UI Drawing
        self.draw_main_ui(egui_context);

        // Create Debug printer
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

        self.draw_esp_test(painter);

        // Input Handling
        if egui_context.wants_pointer_input() || egui_context.wants_keyboard_input() {
            glfw_backend.set_passthrough(false);
        } else {
            glfw_backend.set_passthrough(true)
        }

        let end = std::time::Instant::now();
        let duration = end.duration_since(start);
        println!("Drawing UI took: {:?}", duration);

        egui_context.request_repaint();

        self.frames = self.frames + 1;
        if self.frames == 10 {
            flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap();
        }
    }
}
