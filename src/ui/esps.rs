use egui::{Color32, Painter, Rect, Stroke, Vec2};

use crate::{
    app,
    unreal::{
        global::get_process,
        offsets::{LOCALPLAYERS, OWNING_GAME_INSTANCE, PLAYER_CONTROLLER_OFFSET},
        screen::{Vector2, world2screen},
        types::structs::{FMinimalViewInfo, FName, FString, FVector, TArray, UObject},
    },
};

impl app::App {
    pub fn draw_line(&self, painter: &Painter, start: Vector2, end: Vector2) {
        painter.line_segment(
            [start.to_egui(), end.to_egui()],
            Stroke::new(1.0, Color32::WHITE),
        );
    }

    pub fn draw_box(&self, painter: &Painter, top_left: Vector2, width: f32, height: f32) {
        let box_mode = 2; // TODO: Make this configurable

        match box_mode {
            1 => {
                let top_right = Vector2 {
                    x: top_left.x + width,
                    y: top_left.y,
                };
                let bottom_left = Vector2 {
                    x: top_left.x,
                    y: top_left.y + height,
                };
                let bottom_right = Vector2 {
                    x: top_left.x + width,
                    y: top_left.y + height,
                };

                self.draw_line(painter, top_left, top_right);
                self.draw_line(painter, top_right, bottom_right);
                self.draw_line(painter, bottom_right, bottom_left);
                self.draw_line(painter, bottom_left, top_left);
            }
            // 2, almost the same as 1 but the connection lines have some space between them
            2 => {
                let top_right = Vector2 {
                    x: top_left.x + width,
                    y: top_left.y,
                };
                let bottom_left = Vector2 {
                    x: top_left.x,
                    y: top_left.y + height,
                };
                let bottom_right = Vector2 {
                    x: top_left.x + width,
                    y: top_left.y + height,
                };

                let line_len = width / 2.5;

                // Top horizontal lines
                self.draw_line(
                    painter,
                    top_left,
                    Vector2 {
                        x: top_left.x + line_len,
                        y: top_left.y,
                    },
                );
                self.draw_line(
                    painter,
                    Vector2 {
                        x: top_right.x - line_len,
                        y: top_right.y,
                    },
                    top_right,
                );

                // Bottom horizontal lines
                self.draw_line(
                    painter,
                    bottom_left,
                    Vector2 {
                        x: bottom_left.x + line_len,
                        y: bottom_left.y,
                    },
                );
                self.draw_line(
                    painter,
                    Vector2 {
                        x: bottom_right.x - line_len,
                        y: bottom_right.y,
                    },
                    bottom_right,
                );

                // Left vertical lines
                self.draw_line(
                    painter,
                    top_left,
                    Vector2 {
                        x: top_left.x,
                        y: top_left.y + line_len,
                    },
                );
                self.draw_line(
                    painter,
                    Vector2 {
                        x: bottom_left.x,
                        y: bottom_left.y - line_len,
                    },
                    bottom_left,
                );

                // Right vertical lines
                self.draw_line(
                    painter,
                    top_right,
                    Vector2 {
                        x: top_right.x,
                        y: top_right.y + line_len,
                    },
                );
                self.draw_line(
                    painter,
                    Vector2 {
                        x: bottom_right.x,
                        y: bottom_right.y - line_len,
                    },
                    bottom_right,
                );
            }
            _ => {}
        }
    }

    pub fn draw_player_esp(&mut self, painter: Painter) {
        let proc = &self.process;

        if self.playercam == 0 || self.gamestate == 0 {
            return;
        }
        let min_view_info: FMinimalViewInfo = proc.read(self.playercam + 0x13a0 + 0x10).unwrap();

        let player_array: TArray<usize> = proc.read(self.gamestate + 0x368).unwrap();
        let players = player_array.read_all::<usize>().unwrap();
        for ply in players.iter() {
            let player_life_state = match proc.read::<u8>(ply + 0x620) {
                Ok(val) => val,
                Err(_) => continue,
            };

            let pawn: usize = proc.read(ply + 0x3d8).unwrap();
            if pawn == 0 {
                continue;
            }

            let pawn_root: usize = proc.read(pawn + 0x240).unwrap();
            let player_name_fname = proc.read::<FString>(ply + 0x460).unwrap();
            let player_name = player_name_fname.to_str();

            let pawn_location: FVector = proc.read(pawn_root + 0x1b8).unwrap();

            let mut head_position = pawn_location.clone();
            head_position.z += 85.0;

            let mut head_screen_pos = world2screen(head_position, min_view_info);
            head_screen_pos.x -= 25.0;

            let ground_screen_pos = world2screen(pawn_location, min_view_info);

            let base_height = (ground_screen_pos.y - head_screen_pos.y).abs();
            let width = base_height / 0.8;
            head_screen_pos.x -= width / 2.5;

            self.draw_box(&painter.clone(), head_screen_pos, base_height, width);
            head_screen_pos.y -= 5.0;

            let player_state_str = match player_life_state {
                0 => "Alive",
                _ => "Dead",
            };

            painter.text(
                head_screen_pos.to_egui(),
                egui::Align2::LEFT_BOTTOM,
                format!("{} : {}", player_name, player_state_str),
                egui::FontId::default(),
                Color32::WHITE,
            );
        }
    }

    pub fn draw_spice_field(
        &mut self,
        painter: Painter,
        spice_field: usize,
        min_view_info: FMinimalViewInfo,
    ) {
        let proc = &self.process;

        let pawn_root: usize = proc.read(spice_field + 0x240).unwrap();
        if pawn_root == 0 {
            return;
        };

        let pawn_location: FVector = proc.read(pawn_root + 0x1b8).unwrap();
        let ground_screen_pos = world2screen(pawn_location, min_view_info);

        let spice_field_type = proc.read::<FName>(spice_field + 0x690).unwrap();

        let status = proc.read::<u32>(spice_field + 0x750).unwrap();
        if status == 3 {
            return;
        }

        let status_text = match status {
            0 => "Soon",
            1 => "Active",
            2 => "Active",
            _ => "Unknown",
        };

        let distance = pawn_location.distance_meter(min_view_info.location);

        //println!("Spice Field Type {}", spice_field_type.to_str());
        painter.text(
            ground_screen_pos.to_egui(),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "{} Spice Field | {} | {}m",
                spice_field_type.to_str(),
                status_text,
                distance.floor()
            ),
            egui::FontId::default(),
            Color32::WHITE,
        );
    }

    pub fn draw_npc_esp(
        &mut self,
        painter: Painter,
        actor: usize,
        min_view_info: FMinimalViewInfo,
    ) {
        let proc = &self.process;
        let pawn_root: usize = proc.read(actor + 0x240).unwrap();
        if pawn_root == 0 {
            return;
        };

        let pawn_location: FVector = proc.read(pawn_root + 0x1b8).unwrap();

        let mut head_position = pawn_location.clone();
        head_position.z += 85.0;

        let mut head_screen_pos = world2screen(head_position, min_view_info);

        let ground_screen_pos = world2screen(pawn_location, min_view_info);

        // Make height proportional to distance
        let base_height = (ground_screen_pos.y - head_screen_pos.y).abs();
        let width = base_height / 0.8;
        head_screen_pos.x -= width / 2.0;

        self.draw_box(&painter.clone(), head_screen_pos, base_height, width);
    }

    pub fn draw_esp_test(&mut self, painter: Painter) {
        if self.only_player_esp {
            return;
        }

        if self.playercam == 0 {
            return;
        }

        let min_view_info: FMinimalViewInfo =
            self.process.read(self.playercam + 0x13a0 + 0x10).unwrap();

        let actors: Vec<usize> = self.actors.clone();

        for actor in actors.iter() {
            if *actor == 0 {
                continue;
            }

            let actor_uobject = self.process.read::<UObject>(*actor);
            if actor_uobject.is_err() {
                continue;
            }
            let actor_uobject = actor_uobject.unwrap();
            let actor_name = actor_uobject.get_name();

            if actor_name.starts_with("BP_Npc_SoldierBase") && self.npc_esp {
                self.draw_npc_esp(painter.clone(), *actor, min_view_info);
            } else if actor_name.starts_with("BP_SpiceField") && self.spice_esp {
                self.draw_spice_field(painter.clone(), *actor, min_view_info);
            } else {
                continue;
            }
        }
    }
}
