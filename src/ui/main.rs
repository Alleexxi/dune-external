use egui::{Color32, Painter, Stroke};

use crate::{
    app,
    unreal::{
        global::get_process,
        offsets::{LOCALPLAYERS, OWNING_GAME_INSTANCE, PLAYER_CONTROLLER_OFFSET},
        screen::world2screen,
        types::structs::{FMinimalViewInfo, FName, FVector, TArray},
    },
};

impl app::App {
    pub fn draw_main_ui(&mut self, egui_context: &egui::Context) {
        egui::Window::new("controls").show(egui_context, |ui| {
            ui.set_width(300.0);
            ui.label(format!("current frame number: {}", 0));
        });
    }
    #[flamer::flame]
    pub fn draw_esp_test(&mut self, painter: Painter) {
        let proc = get_process();

        let min_view_info: FMinimalViewInfo = proc.read(self.playercam + 0x13a0 + 0x10).unwrap();
        //let player_array: TArray<usize> = proc.read(self.gamestate + 0x368).unwrap();

        //let players = player_array.read_all::<usize>().unwrap();
        //for ply in players.iter() {
        //    let pawn: usize = proc.read(ply + 0x3d8).unwrap();
        //    if pawn == 0 {
        //        continue;
        //    }
        //
        //    let pawn_root: usize = proc.read(pawn + 0x240).unwrap();
        //    let pawn_location: FVector = proc.read(pawn_root + 0x1b8).unwrap();
        //
        //    let screen_pos = world2screen(pawn_location, min_view_info);
        //
        //    painter.circle(screen_pos.to_egui(), 5.0, Color32::DARK_GRAY, Stroke::NONE);
        //}
        for actor in self.actors.iter() {
            if actor == &0 {
                continue;
            }

            let actor_name: String = proc.read::<FName>(actor + 0x18).unwrap().to_string();

            let pawn_root: usize = proc.read(actor + 0x240).unwrap();
            if pawn_root == 0 {
                continue;
            };
            let pawn_location: FVector = proc.read(pawn_root + 0x1b8).unwrap();

            let screen_pos = world2screen(pawn_location, min_view_info);
            if screen_pos.x == 0.0 && screen_pos.y == 0.0 {
                continue;
            }
            painter.circle(screen_pos.to_egui(), 5.0, Color32::DARK_GRAY, Stroke::NONE);
        }
    }
}
