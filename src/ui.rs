use eframe::egui;
use log::error;

use crate::docker::{ContainerInfo, DockerClient};

pub const WINDOW_WIDTH: f32 = 520.0;
pub const DEFAULT_HEIGHT: f32 = 240.0;
const HEADER_HEIGHT: f32 = 32.0;
const ROW_HEIGHT: f32 = 28.0;
const WINDOW_PADDING: f32 = 24.0;

pub struct MyApp {
    docker: DockerClient,
    containers: Vec<ContainerInfo>,
    error: Option<String>,
    window_height: f32,
    pending_resize: Option<f32>,
}

impl MyApp {
    pub fn new() -> Self {
        let docker = DockerClient::new();
        let mut app = Self {
            docker,
            containers: Vec::new(),
            error: None,
            window_height: 0.0,
            pending_resize: None,
        };
        app.refresh_containers();
        app
    }

    fn refresh_containers(&mut self) {
        match self.docker.list_containers() {
            Ok(containers) => {
                self.error = None;
                self.containers = containers;
                self.update_window_height();
            }
            Err(err) => {
                error!("failed to list containers: {err}");
                self.error = Some(format!("Failed to list containers: {err}"));
            }
        }
    }

    fn update_window_height(&mut self) {
        let target_height =
            (HEADER_HEIGHT + ROW_HEIGHT * self.containers.len() as f32 + WINDOW_PADDING)
                .max(200.0);
        if self.window_height == 0.0 || target_height > self.window_height {
            self.window_height = target_height;
            self.pending_resize = Some(target_height);
        }
    }

    fn start_container(&mut self, id: &str) {
        if let Err(err) = self.docker.start_container(id) {
            error!("failed to start container: {err}");
            self.error = Some(format!("Failed to start container: {err}"));
            return;
        }
        self.refresh_containers();
    }

    fn stop_container(&mut self, id: &str) {
        if let Err(err) = self.docker.stop_container(id) {
            error!("failed to stop container: {err}");
            self.error = Some(format!("Failed to stop container: {err}"));
            return;
        }
        self.refresh_containers();
    }

    fn restart_container(&mut self, id: &str) {
        if let Err(err) = self.docker.restart_container(id) {
            error!("failed to restart container: {err}");
            self.error = Some(format!("Failed to restart container: {err}"));
            return;
        }
        self.refresh_containers();
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(height) = self.pending_resize.take() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                    WINDOW_WIDTH,
                    height,
                )));
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("close").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if ui.button("refresh").clicked() {
                    self.refresh_containers();
                }
            });

            if let Some(message) = &self.error {
                ui.colored_label(egui::Color32::RED, message);
            }

            let containers = self.containers.clone();
            for container in containers {
                let is_stopped = matches!(container.state.as_str(), "exited" | "dead" | "created");
                let mut action: Option<ContainerAction> = None;
                ui.horizontal(|ui| {
                    if ui.button("stop").clicked() {
                        action = Some(ContainerAction::Stop(container.id.clone()));
                    }
                    if ui.button("start").clicked() {
                        action = Some(ContainerAction::Start(container.id.clone()));
                    }
                    if !is_stopped {
                        if ui.button("restart").clicked() {
                            action = Some(ContainerAction::Restart(container.id.clone()));
                        }
                        let _ = ui.button(" "); // exec placeholder
                    }

                    let color = match container.state.as_str() {
                        "running" => egui::Color32::LIGHT_GREEN,
                        "exited" => egui::Color32::LIGHT_RED,
                        _ => egui::Color32::KHAKI,
                    };
                    ui.colored_label(color, &container.state);
                    ui.label(&container.name);
                });

                if let Some(action) = action {
                    match action {
                        ContainerAction::Start(id) => self.start_container(&id),
                        ContainerAction::Stop(id) => self.stop_container(&id),
                        ContainerAction::Restart(id) => self.restart_container(&id),
                    }
                }
            }
        });
    }
}

#[derive(Clone, Debug)]
enum ContainerAction {
    Start(String),
    Stop(String),
    Restart(String),
}
