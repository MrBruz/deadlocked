use eframe::egui::{self, Align2, Layout, Ui};
use std::sync::mpsc;
use strum::IntoEnumIterator;

use crate::{
    colors::Colors,
    config::{parse_config, write_config, AimbotConfig, AimbotStatus, Config},
    key_codes::KeyCode,
    message::{Game, Message},
    mouse::MouseStatus,
};

pub struct Gui {
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    config: Config,
    status: AimbotStatus,
    mouse_status: MouseStatus,
}

impl Gui {
    pub fn new(tx: mpsc::Sender<Message>, rx: mpsc::Receiver<Message>) -> Self {
        // read config
        let config = parse_config();
        let status = AimbotStatus::GameNotStarted;
        let out = Self {
            tx,
            rx,
            config,
            status,
            mouse_status: MouseStatus::NoMouseFound,
        };
        write_config(&out.config);
        out
    }

    fn send_message(&self, message: Message) {
        if let Err(error) = self.tx.send(message) {
            println!("{}", error);
        }
    }

    fn add_grid(&mut self, ui: &mut Ui) {
        let mut game_config = self
            .config
            .games
            .get_mut(&self.config.current_game)
            .unwrap()
            .clone();

        egui::Grid::new("cs2")
            .num_columns(2)
            .min_col_width(75.0)
            .show(ui, |ui| {
                ui.label("Enable Aimbot")
                    .on_hover_text("general aimbot enable");
                if ui.checkbox(&mut game_config.aimbot, "").changed() {
                    self.send_message(Message::ConfigEnableAimbot(game_config.aimbot));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Hotkey")
                    .on_hover_text("which key or mouse button should activate the aimbot");
                egui::ComboBox::new("cs2_hotkey", "")
                    .selected_text(format!("{:?}", game_config.hotkey))
                    .show_ui(ui, |ui| {
                        for key_code in KeyCode::iter() {
                            let text = format!("{:?}", &key_code);
                            if ui
                                .selectable_value(&mut game_config.hotkey, key_code, text)
                                .clicked()
                            {
                                self.send_message(Message::ConfigHotkey(game_config.hotkey));
                                self.write_game_config(&game_config);
                            }
                        }
                    });
                ui.end_row();

                ui.label("Start Bullet")
                    .on_hover_text("after how many bullets fired in a row the aimbot should start");
                if ui
                    .add(egui::Slider::new(&mut game_config.start_bullet, 0..=5))
                    .changed()
                {
                    self.send_message(Message::ConfigStartBullet(game_config.start_bullet));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Aim Lock")
                    .on_hover_text("whether the aim should lock onto the target");
                if ui.checkbox(&mut game_config.aim_lock, "").changed() {
                    self.send_message(Message::ConfigAimLock(game_config.aim_lock));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Visibility Check")
                    .on_hover_text("whether to check for player visibility");
                if ui.checkbox(&mut game_config.visibility_check, "").changed() {
                    self.send_message(Message::ConfigVisibilityCheck(game_config.visibility_check));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("FOV")
                    .on_hover_text("how much around the crosshair the aimbot should \"see\"");
                if ui
                    .add(
                        egui::Slider::new(&mut game_config.fov, 0.1..=10.0)
                            .suffix("°")
                            .step_by(0.1),
                    )
                    .changed()
                {
                    self.send_message(Message::ConfigFOV(game_config.fov));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Smooth")
                    .on_hover_text("how much the aimbot input should be smoothed, higher is more");
                if ui
                    .add(egui::Slider::new(&mut game_config.smooth, 1.0..=10.0))
                    .changed()
                {
                    self.send_message(Message::ConfigSmooth(game_config.smooth));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Multibone").on_hover_text(
                    "whether the aimbot should aim at all of the body, or just the head",
                );
                if ui.checkbox(&mut game_config.multibone, "").changed() {
                    self.send_message(Message::ConfigMultibone(game_config.multibone));
                    self.write_game_config(&game_config);
                }
                ui.end_row();

                ui.label("Enable RCS").on_hover_text(
                    "whether recoil should be compensated when the aimbot is not active",
                );
                if ui.checkbox(&mut game_config.rcs, "").changed() {
                    self.send_message(Message::ConfigEnableRCS(game_config.rcs));
                    self.write_game_config(&game_config);
                }
            });

        *self
            .config
            .games
            .get_mut(&self.config.current_game)
            .unwrap() = game_config.clone();
    }

    fn add_game_status(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(self.status.string()).heading().color(
                match self.status {
                    AimbotStatus::Working => Colors::GREEN,
                    AimbotStatus::GameNotStarted => Colors::YELLOW,
                },
            ));

            let mouse_text = match &self.mouse_status {
                MouseStatus::Working(name) => name,
                MouseStatus::PermissionsRequired => {
                    "mouse input only works when user is in input group"
                }
                MouseStatus::Disconnected => "mouse was disconnected",
                MouseStatus::NoMouseFound => "no mouse was found",
            };
            let color = if let MouseStatus::Working(_) = &self.mouse_status {
                egui::Color32::PLACEHOLDER
            } else {
                Colors::YELLOW
            };
            ui.label(egui::RichText::new(mouse_text).color(color));
        });
    }

    fn write_game_config(&self, game_config: &AimbotConfig) {
        let mut config = self.config.clone();
        *config.games.get_mut(&self.config.current_game).unwrap() = game_config.clone();
        write_config(&config);
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        // makes it more inefficient to force draw 60fps, but else the mouse disconnect message does not show up
        // todo: when update is split into tick and show, put message parsing into tick and force update the ui when message are received
        ctx.request_repaint();

        while let Ok(message) = self.rx.try_recv() {
            match message {
                Message::Status(status) => self.status = status,
                Message::MouseStatus(status) => self.mouse_status = status,
                _ => {}
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                egui::ComboBox::new("game", "Current Game")
                    .selected_text(self.config.current_game.string())
                    .show_ui(ui, |ui| {
                        for game in Game::iter() {
                            let text = game.string();
                            if ui
                                .selectable_value(&mut self.config.current_game, game.clone(), text)
                                .clicked()
                            {
                                self.send_message(Message::ChangeGame(
                                    self.config.current_game.clone(),
                                ));
                                write_config(&self.config);
                            }
                        }
                    });

                if ui.button("Report Issues").clicked() {
                    ctx.open_url(egui::OpenUrl {
                        url: String::from("https://github.com/avitran0/deadlocked/issues"),
                        new_tab: false,
                    });
                }
            });

            ui.separator();

            egui::Grid::new("main_grid")
                .num_columns(2)
                .spacing([20.0, 5.0])
                .show(ui, |ui| {
                    self.add_grid(ui);
                    self.add_game_status(ui);
                });
        });

        let version = format!(
            "version: {}",
            option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")
        );
        let font = egui::FontId::proportional(12.0);
        let text_size = ctx.fonts(|fonts| {
            fonts
                .layout_no_wrap(version.clone(), font.clone(), egui::Color32::WHITE)
                .size()
        });

        ctx.layer_painter(egui::LayerId::background()).text(
            Align2::RIGHT_BOTTOM
                .align_size_within_rect(text_size, ctx.screen_rect().shrink(4.0))
                .max,
            Align2::RIGHT_BOTTOM,
            version,
            font,
            egui::Color32::GRAY,
        );
    }
}
