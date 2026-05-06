use eframe::egui;
use log::{error, info};

use crate::docker::DockerClient;
use bollard::config::ContainerSummary;
use bollard::config::ContainerSummaryStateEnum;

pub const WINDOW_WIDTH: f32 = 420.0;
pub const DEFAULT_HEIGHT: f32 = 80.0;
const HEADER_HEIGHT: f32 = 32.0;
const ROW_HEIGHT: f32 = 15.0;
const WINDOW_PADDING: f32 = 24.0;

pub struct MyApp {
    docker: DockerClient,
    containers: Vec<ContainerSummary>,
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
                info!("Found {} containers", self.containers.len());
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
            (HEADER_HEIGHT + ROW_HEIGHT * self.containers.len() as f32 + WINDOW_PADDING).max(200.0);
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
        info!("Started container: {id}");
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
        info!("Restarted container: {id}");
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
                let Some(id) = &container.id else {
                    error!("Failed to get container ID");
                    continue;
                };
                let Some(state) = container.state else {
                    error!("Failed to get container state");
                    continue;
                };
                let name = container.names.unwrap_or_default()
                    .get(0)
                    .map(|value| value.trim_start_matches('/').to_string())
                    .unwrap_or_else(|| id.chars().take(12).collect());

                let is_stopped = matches!(state, ContainerSummaryStateEnum::EXITED | ContainerSummaryStateEnum::DEAD | ContainerSummaryStateEnum::CREATED);
                let mut action: Option<ContainerAction> = None;
                ui.horizontal(|ui| {
                    if ui.button("stop").clicked() {
                        action = Some(ContainerAction::Stop(id.clone()));
                    }
                    if ui.button("start").clicked() {
                        action = Some(ContainerAction::Start(id.clone()));
                    }
                    if !is_stopped {
                        if ui.button("restart").clicked() {
                            action = Some(ContainerAction::Restart(id.clone()));
                        }
                        let _ = ui.button("exec"); // exec placeholder
                    } else {
                        let _ = ui.button("             ");
                        let _ = ui.button("         ");
                    }
                    ui.label(name);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let color = match state {
                            ContainerSummaryStateEnum::RUNNING => egui::Color32::LIGHT_GREEN,
                            ContainerSummaryStateEnum::EXITED => egui::Color32::LIGHT_RED,
                            _ => egui::Color32::KHAKI,
                        };
                        ui.colored_label(color, state.as_ref());
                    });
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
enum ContainerAction<> {
    Start(String),
    Stop(String),
    Restart(String),
}
