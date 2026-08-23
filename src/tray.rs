use std::sync::mpsc::Sender;

use eframe::egui;
use tray_icon::{
    Icon as TrayImage, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuId, MenuItem},
};

use crate::constants::{APP_NAME, HOTKEY_LABEL};
use crate::icon::make_icon_rgba;
use crate::model::AppEvent;

pub(crate) fn create(sender: Sender<AppEvent>, ctx: egui::Context) -> Result<TrayIcon, String> {
    let show_item = MenuItem::new("显示窗口", true, None);
    let exit_item = MenuItem::new("退出", true, None);
    let show_id = show_item.id().clone();
    let exit_id = exit_item.id().clone();

    let menu = Menu::with_items(&[&show_item, &exit_item])
        .map_err(|error| format!("无法创建托盘菜单：{error}"))?;

    let (rgba, width, height) = make_icon_rgba();
    let icon = TrayImage::from_rgba(rgba, width, height)
        .map_err(|error| format!("无法创建托盘图标：{error}"))?;

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(format!("{APP_NAME} - {HOTKEY_LABEL}"))
        .with_icon(icon)
        .build()
        .map_err(|error| format!("无法创建托盘图标：{error}"))?;

    install_menu_handler(sender, ctx, show_id, exit_id);
    Ok(tray_icon)
}

fn install_menu_handler(
    sender: Sender<AppEvent>,
    ctx: egui::Context,
    show_id: MenuId,
    exit_id: MenuId,
) {
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let app_event = if event.id == show_id {
            Some(AppEvent::Show)
        } else if event.id == exit_id {
            Some(AppEvent::Exit)
        } else {
            None
        };

        if let Some(app_event) = app_event {
            // A minimized window may not receive ordinary redraw events on Windows.
            // Restore it first so both Show and Exit are processed reliably.
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            let _ = sender.send(app_event);
            ctx.request_repaint();
        }
    }));
}
