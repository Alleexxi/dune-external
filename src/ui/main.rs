use egui::{Color32, Painter, Rect, Stroke, Vec2};
use egui_keybind::{Keybind, Shortcut};

use crate::{
    app,
    unreal::offsets::{
        LOCALPLAYERS, OWNING_GAME_INSTANCE, PLAYER_CHARACTER, PLAYER_CONTROLLER_OFFSET,
    },
};

/// Class
/// BP_StorageContainer_C = Storage
/// BP_Totem_C = Subfief
/// Solider = Npc
/// BP_<Class>Ornithopter = Heli
///

impl app::App {
    pub fn draw_main_ui(&mut self, egui_context: &egui::Context) {
        egui::Window::new("real shit").show(egui_context, |ui| {
            ui.set_width(300.0);

            ui.label(format!("FrameTime: {}ms", self.frames).as_str());
            ui.separator();
            ui.checkbox(&mut self.esp_enabled, "Enable ESP");
            ui.checkbox(&mut self.only_player_esp, "Only Player ESP");
            if !self.only_player_esp {
                ui.checkbox(&mut self.npc_esp, "NPC ESP");
                ui.checkbox(&mut self.spice_esp, "Spice ESP");
            }

            ui.separator();
            ui.checkbox(
                &mut self.weapon_damage_enabled,
                "Enable Weapon Damage (To NPCs Only)",
            );
            if self.weapon_damage_enabled {
                ui.add(
                    egui::Slider::new(&mut self.weapon_damage, 0.0..=100000.0)
                        .text("Weapon Damage"),
                );
            }
        });
    }

    pub fn change_weapon_damage(&mut self) {
        if self.weapon_damage_enabled {
            let player_controller_address = match self
                .process
                .read::<usize>(self.uworld + OWNING_GAME_INSTANCE)
                .and_then(|game_instance_address| {
                    self.process
                        .read::<usize>(game_instance_address + LOCALPLAYERS)
                })
                .and_then(|localplayers_address| self.process.read::<usize>(localplayers_address))
                .and_then(|localplayer_address| {
                    self.process
                        .read::<usize>(localplayer_address + PLAYER_CONTROLLER_OFFSET)
                }) {
                Ok(addr) => addr,
                Err(_) => {
                    return;
                }
            };

            let localcharacter = match self
                .process
                .read::<usize>(player_controller_address + PLAYER_CHARACTER)
            {
                Ok(addr) => addr,
                Err(_) => {
                    // Only skip this block, don't return from the whole function
                    return;
                }
            };

            let weapon_actor = match self.process.read::<usize>(localcharacter + 0xfd8) {
                Ok(addr) => addr,
                Err(_) => {
                    // Only skip this block, don't return from the whole function
                    return;
                }
            };

            if let Err(_) = self
                .process
                .write(weapon_actor + 0x4e0 + 0x160, &self.weapon_damage)
            {
                return;
            }
        }
    }
}
