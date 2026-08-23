use std::sync::mpsc::{self, Receiver};

use eframe::egui::{
    self, Color32, RichText, TextStyle, ViewportCommand,
    text::{LayoutJob, TextFormat},
};
use global_hotkey::GlobalHotKeyManager;
use tray_icon::TrayIcon;

use crate::constants::{APP_NAME, HOTKEY_LABEL};
use crate::hotkey;
use crate::model::{AppEvent, WindowInfo};
use crate::tray;

pub(crate) struct WindowInfoApp {
    window_info: Option<WindowInfo>,
    status: String,
    status_is_error: bool,
    events: Receiver<AppEvent>,
    allow_exit: bool,
    _hotkey_manager: Option<GlobalHotKeyManager>,
    _tray_icon: Option<TrayIcon>,
}

impl WindowInfoApp {
    pub(crate) fn new(ctx: &egui::Context) -> Self {
        let (event_sender, event_receiver) = mpsc::channel();
        let mut initialization_errors = Vec::new();

        let hotkey_manager = match hotkey::install(event_sender.clone(), ctx.clone()) {
            Ok(manager) => Some(manager),
            Err(error) => {
                initialization_errors.push(error);
                None
            }
        };

        let tray_icon = match tray::create(event_sender, ctx.clone()) {
            Ok(tray_icon) => Some(tray_icon),
            Err(error) => {
                initialization_errors.push(error);
                None
            }
        };

        let (status, status_is_error) = if initialization_errors.is_empty() {
            (
                format!("工具已在后台运行。激活目标窗口后按 {HOTKEY_LABEL}。"),
                false,
            )
        } else {
            show_window(ctx);
            (initialization_errors.join("\n"), true)
        };

        Self {
            window_info: None,
            status,
            status_is_error,
            events: event_receiver,
            allow_exit: false,
            _hotkey_manager: hotkey_manager,
            _tray_icon: tray_icon,
        }
    }

    fn handle_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                AppEvent::CaptureStarted => {
                    self.status = "正在读取 GlazeWM 的焦点窗口……".to_owned();
                    self.status_is_error = false;
                }
                AppEvent::CaptureFinished(Ok(window_info)) => {
                    self.window_info = Some(window_info);
                    self.status = "查询完成。可用鼠标选择下方文本并复制。".to_owned();
                    self.status_is_error = false;
                    show_window(ctx);
                }
                AppEvent::CaptureFinished(Err(error)) => {
                    self.status = error;
                    self.status_is_error = true;
                    show_window(ctx);
                }
                AppEvent::Show => show_window(ctx),
                AppEvent::Exit => {
                    self.allow_exit = true;
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            }
        }
    }

    fn render_summary(&self, ui: &mut egui::Ui) {
        let Some(window_info) = &self.window_info else {
            ui.label(format!(
                "激活需要检查的窗口，然后按 {HOTKEY_LABEL}。\n查询完成后这里会显示 GlazeWM 实际返回的数据。"
            ));
            return;
        };

        egui::Grid::new("window_summary")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Process");
                ui.add(egui::Label::new(&window_info.process_name).selectable(true));
                ui.end_row();

                ui.label("Class");
                ui.add(egui::Label::new(&window_info.class_name).selectable(true));
                ui.end_row();

                ui.label("Title");
                ui.add(egui::Label::new(&window_info.title).selectable(true));
                ui.end_row();
            });
    }

    fn render_json(&self, ui: &mut egui::Ui) -> bool {
        let Some(window_info) = &self.window_info else {
            return false;
        };

        let copy_requested = ui
            .horizontal(|ui| {
                ui.label("完整 JSON（可选择文本并按 Ctrl+C 复制）");
                ui.button("复制完整 JSON").clicked()
            })
            .inner;

        if copy_requested {
            ui.ctx().copy_text(window_info.formatted_json.clone());
        }

        let available_height = (ui.available_height() - 42.0).max(120.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            egui::ScrollArea::both()
                .max_height(available_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let highlighted_json = json_layout(ui, &window_info.formatted_json);
                    ui.add(egui::Label::new(highlighted_json).selectable(true));
                });
        });

        copy_requested
    }
}

fn json_layout(ui: &egui::Ui, json: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    let font_id = TextStyle::Monospace.resolve(ui.style());
    let normal_color = ui.visuals().text_color();

    let (process_color, class_color, title_color) = if ui.visuals().dark_mode {
        (
            Color32::from_rgb(120, 210, 140),
            Color32::from_rgb(110, 180, 255),
            Color32::from_rgb(245, 190, 100),
        )
    } else {
        (
            Color32::from_rgb(25, 125, 55),
            Color32::from_rgb(25, 90, 180),
            Color32::from_rgb(175, 95, 15),
        )
    };

    for line in json.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let color = if trimmed.starts_with("\"processName\":") {
            process_color
        } else if trimmed.starts_with("\"className\":") {
            class_color
        } else if trimmed.starts_with("\"title\":") {
            title_color
        } else {
            normal_color
        };

        job.append(
            line,
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color,
                ..Default::default()
            },
        );
    }

    job
}

impl eframe::App for WindowInfoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_events(&ctx);

        if ctx.input(|input| input.viewport().close_requested()) && !self.allow_exit {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(APP_NAME);
            ui.label(RichText::new(format!("全局热键：{HOTKEY_LABEL}")).weak());
            ui.add_space(8.0);

            let status_color = if self.status_is_error {
                Color32::from_rgb(220, 80, 80)
            } else {
                ui.visuals().text_color()
            };
            ui.label(RichText::new(&self.status).color(status_color));
            ui.separator();

            self.render_summary(ui);

            if self.window_info.is_some() {
                ui.add_space(8.0);
                ui.separator();
                if self.render_json(ui) {
                    self.status = "完整 JSON 已复制到剪贴板。".to_owned();
                    self.status_is_error = false;
                }
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("隐藏到托盘").clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                }

                if ui.button("退出").clicked() {
                    self.allow_exit = true;
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            });
        });
    }
}

fn show_window(ctx: &egui::Context) {
    ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
    ctx.send_viewport_cmd(ViewportCommand::Visible(true));
    ctx.send_viewport_cmd(ViewportCommand::Focus);
}
