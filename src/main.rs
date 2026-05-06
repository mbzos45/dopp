use bollard::errors::Error as BollardError;
use bollard::query_parameters::{
    ListContainersOptionsBuilder, RestartContainerOptionsBuilder, StartContainerOptions,
    StopContainerOptionsBuilder,
};
use bollard::Docker;
use eframe::egui;
use log::error;

const WINDOW_WIDTH: f32 = 520.0;
const HEADER_HEIGHT: f32 = 32.0;
const ROW_HEIGHT: f32 = 28.0;
const WINDOW_PADDING: f32 = 24.0;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([WINDOW_WIDTH, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "dopp",
        options,
        Box::new(|_| Ok(Box::new(MyApp::new()))),
    )
}

#[derive(Clone, Debug)]
struct ContainerInfo {
    id: String,
    name: String,
    state: String,
}

struct MyApp {
    docker: Option<Docker>,
    runtime: tokio::runtime::Runtime,
    containers: Vec<ContainerInfo>,
    error: Option<String>,
    window_height: f32,
    pending_resize: Option<f32>,
}

impl MyApp {
    fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");
        let docker = Docker::connect_with_local_defaults().ok();
        let mut app = Self {
            docker,
            runtime,
            containers: Vec::new(),
            error: None,
            window_height: 0.0,
            pending_resize: None,
        };
        app.refresh_containers();
        app
    }

    fn refresh_containers(&mut self) {
        let Some(docker) = &self.docker else {
            self.error = Some("Docker connection is unavailable".to_string());
            return;
        };

        match self.runtime.block_on(list_containers(docker)) {
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
        let target_height = (HEADER_HEIGHT + ROW_HEIGHT * self.containers.len() as f32 + WINDOW_PADDING)
            .max(200.0);
        if self.window_height == 0.0 || target_height > self.window_height {
            self.window_height = target_height;
            self.pending_resize = Some(target_height);
        }
    }

    fn start_container(&mut self, id: &str) {
        if let Some(docker) = &self.docker {
            let result = self
                .runtime
                .block_on(docker.start_container(id, None::<StartContainerOptions>));
            if let Err(err) = result {
                error!("failed to start container: {err}");
                self.error = Some(format!("Failed to start container: {err}"));
                return;
            }
            self.refresh_containers();
        }
    }

    fn stop_container(&mut self, id: &str) {
        if let Some(docker) = &self.docker {
            let options = StopContainerOptionsBuilder::default().t(10).build();
            let result = self.runtime.block_on(docker.stop_container(id, Some(options)));
            if let Err(err) = result {
                error!("failed to stop container: {err}");
                self.error = Some(format!("Failed to stop container: {err}"));
                return;
            }
            self.refresh_containers();
        }
    }

    fn restart_container(&mut self, id: &str) {
        if let Some(docker) = &self.docker {
            let options = RestartContainerOptionsBuilder::default().t(10).build();
            let result = self
                .runtime
                .block_on(docker.restart_container(id, Some(options)));
            if let Err(err) = result {
                error!("failed to restart container: {err}");
                self.error = Some(format!("Failed to restart container: {err}"));
                return;
            }
            self.refresh_containers();
        }
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

async fn list_containers(docker: &Docker) -> Result<Vec<ContainerInfo>, BollardError> {
    let options = ListContainersOptionsBuilder::default().all(true).build();
    let containers = docker.list_containers(Some(options)).await?;
    let mut result = Vec::new();
    for container in containers {
        let id = container.id.unwrap_or_default();
        let name = container
            .names
            .unwrap_or_default()
            .get(0)
            .map(|value| value.trim_start_matches('/').to_string())
            .unwrap_or_else(|| id.chars().take(12).collect());
        let state = container
            .state
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        result.push(ContainerInfo { id, name, state });
    }
    Ok(result)
}
