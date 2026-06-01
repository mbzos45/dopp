#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![expect(rustdoc::missing_crate_level_docs)]

mod docker;
mod terminal;

use eframe::egui;
use eframe::egui::IconData;
use log::{error, info};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::docker::{ContainerActionKind, DockerRunner, UiEvent};
use crate::terminal::{ExecCommand, ExecOptions};
use bollard::config::ContainerSummary;
use bollard::config::ContainerSummaryStateEnum;

const WINDOW_WIDTH: f32 = 420.0;
const DEFAULT_HEIGHT: f32 = 80.0;
const HEADER_HEIGHT: f32 = 32.0;
const ROW_HEIGHT: f32 = 15.0;
const WINDOW_PADDING: f32 = 24.0;

struct MyApp {
    runner: DockerRunner,
    events: mpsc::UnboundedReceiver<UiEvent>,
    containers: Vec<ContainerSummary>,
    error: Option<String>,
    window_height: f32,
    pending_resize: Option<f32>,
    loading_containers: HashSet<String>,
    needs_initial_refresh: bool,
    settings: AppSettings,
}

#[derive(Clone, Debug)]
struct AppSettings {
    copy_command_to_clipboard: bool,
    inherit_xauthority_on_linux: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            copy_command_to_clipboard: true,
            inherit_xauthority_on_linux: true,
        }
    }
}

impl MyApp {
    fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let runner = DockerRunner::new(tx);
        Self {
            runner,
            events: rx,
            containers: Vec::new(),
            error: None,
            window_height: 0.0,
            pending_resize: None,
            loading_containers: HashSet::new(),
            needs_initial_refresh: true,
            settings: AppSettings::default(),
        }
    }

    fn spawn_refresh(&self, ctx: &egui::Context) {
        self.runner.spawn_refresh(ctx.clone());
    }

    fn spawn_action(&mut self, action: ContainerActionKind, id: String, ctx: &egui::Context) {
        self.loading_containers.insert(id.clone());
        self.runner.spawn_action(id, action, ctx.clone());
    }

    fn drain_events(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                UiEvent::ContainersRefreshed(result) => match result {
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
                },
                UiEvent::ContainerActionFinished { id, action, result } => {
                    self.loading_containers.remove(&id);
                    if let Err(err) = result {
                        error!("failed to {} container: {err}", action.as_str());
                        self.error =
                            Some(format!("Failed to {} container: {err}", action.as_str()));
                    }
                }
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
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.needs_initial_refresh {
            self.spawn_refresh(ui.ctx());
            self.needs_initial_refresh = false;
        }

        self.drain_events();

        if let Some(height) = self.pending_resize.take() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                    WINDOW_WIDTH,
                    height,
                )));
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.settings.copy_command_to_clipboard, "copy command");
                ui.checkbox(
                    &mut self.settings.inherit_xauthority_on_linux,
                    "XAUTHORITY on Linux",
                );
            });

            ui.horizontal(|ui| {
                if ui.button("close").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if ui.button("refresh").clicked() {
                    self.spawn_refresh(ui.ctx());
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
                let name = container
                    .names
                    .unwrap_or_default()
                    .get(0)
                    .map(|value| value.trim_start_matches('/').to_string())
                    .unwrap_or_else(|| id.chars().take(12).collect());

                let is_stopped = matches!(
                    state,
                    ContainerSummaryStateEnum::EXITED
                        | ContainerSummaryStateEnum::DEAD
                        | ContainerSummaryStateEnum::CREATED
                );
                let is_loading = self.loading_containers.contains(id);
                let mut action: Option<ContainerActionKind> = None;
                let mut exec_requested = false;
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!is_loading, egui::Button::new("stop"))
                        .clicked()
                    {
                        action = Some(ContainerActionKind::Stop);
                    }
                    if ui
                        .add_enabled(!is_loading, egui::Button::new("start"))
                        .clicked()
                    {
                        action = Some(ContainerActionKind::Start);
                    }
                    if !is_stopped {
                        if ui
                            .add_enabled(!is_loading, egui::Button::new("restart"))
                            .clicked()
                        {
                            action = Some(ContainerActionKind::Restart);
                        }
                        if ui
                            .add_enabled(!is_loading, egui::Button::new("exec"))
                            .clicked()
                        {
                            exec_requested = true;
                        }
                    } else {
                        let _ = ui.add_enabled(false, egui::Button::new("             "));
                        let _ = ui.add_enabled(false, egui::Button::new("         "));
                    }
                    if is_loading {
                        ui.add(egui::Spinner::new());
                    }
                    ui.label(&name);
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
                    info!("{}: {name}", action.as_str());
                    self.spawn_action(action, id.clone(), ui.ctx());
                }
                if exec_requested {
                    info!("exec: {name}");
                    let exec_options = ExecOptions {
                        inherit_xauthority_on_linux: self.settings.inherit_xauthority_on_linux,
                    };
                    let exec_command = ExecCommand::new(&name, exec_options);
                    let exec_string = exec_command.as_string();

                    #[cfg(not(target_os = "macos"))]
                    if let Err(err) = terminal::launch_exec_terminal(&exec_command) {
                        error!("exec terminal launch failed: {err}");
                        self.error = Some(format!("Failed to launch terminal: {err}"));
                    }

                    // macOS ではターミナル起動がないためクリップボードへのコピーは必ず実行する
                    #[cfg(not(target_os = "macos"))]
                    let should_copy = self.settings.copy_command_to_clipboard;
                    #[cfg(target_os = "macos")]
                    let should_copy = true;

                    if should_copy {
                        if let Err(clip_err) = terminal::copy_to_clipboard(&exec_string) {
                            error!("clipboard copy failed: {clip_err}");
                        }
                    }
                }
            }
        });
    }
}

fn main() -> eframe::Result {
    let icon_data = load_icon(include_bytes!("../assets/icon_32x32.png"));

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let mut options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, DEFAULT_HEIGHT]),
        ..Default::default()
    };
    if let Some(icon) = icon_data {
        options.viewport.icon = Some(Arc::new(icon));
    }
    eframe::run_native(
        env!("CARGO_PKG_NAME"),
        options,
        Box::new(|_| Ok(Box::new(MyApp::new()))),
    )
}

fn load_icon(bytes: &[u8]) -> Option<IconData> {
    let image = match image::load_from_memory(bytes) {
       Ok(x) => x.into_rgba8(),
       Err(_) => return None,
    };
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Some(IconData {
       rgba,
       width,
       height,
    })
 }
