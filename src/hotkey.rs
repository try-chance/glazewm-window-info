use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
};
use std::thread;

use eframe::egui;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};

use crate::constants::HOTKEY_LABEL;
use crate::glazewm::query_focused_window;
use crate::model::AppEvent;

pub(crate) fn install(
    sender: Sender<AppEvent>,
    ctx: egui::Context,
) -> Result<GlobalHotKeyManager, String> {
    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyI);
    let hotkey_id = hotkey.id();
    let manager =
        GlobalHotKeyManager::new().map_err(|error| format!("无法初始化全局热键：{error}"))?;
    manager
        .register(hotkey)
        .map_err(|error| format!("无法注册全局热键 {HOTKEY_LABEL}：{error}"))?;

    let capture_in_progress = Arc::new(AtomicBool::new(false));
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        if event.id != hotkey_id || event.state != HotKeyState::Pressed {
            return;
        }

        if capture_in_progress.swap(true, Ordering::AcqRel) {
            return;
        }

        let _ = sender.send(AppEvent::CaptureStarted);
        ctx.request_repaint();

        let sender = sender.clone();
        let ctx = ctx.clone();
        let capture_in_progress = capture_in_progress.clone();
        thread::spawn(move || {
            let result = query_focused_window();
            capture_in_progress.store(false, Ordering::Release);
            let _ = sender.send(AppEvent::CaptureFinished(result));
            ctx.request_repaint();
        });
    }));

    Ok(manager)
}
