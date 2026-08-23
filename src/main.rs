#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod constants;
mod fonts;
mod glazewm;
mod hotkey;
mod icon;
mod model;
mod tray;

use std::sync::Arc;

use eframe::egui::{IconData, ViewportBuilder};
use single_instance::SingleInstance;

use crate::app::WindowInfoApp;
use crate::constants::{APP_NAME, INSTANCE_NAME};
use crate::fonts::configure_fonts;
use crate::icon::make_icon_rgba;

fn main() -> eframe::Result {
    let instance =
        SingleInstance::new(INSTANCE_NAME).expect("failed to create the single-instance mutex");
    if !instance.is_single() {
        return Ok(());
    }

    let (icon_rgba, icon_width, icon_height) = make_icon_rgba();
    let viewport_icon = Arc::new(IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    });

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_inner_size([700.0, 500.0])
            .with_min_inner_size([560.0, 420.0])
            .with_always_on_top()
            .with_active(false)
            .with_visible(false)
            .with_icon(viewport_icon),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|creation_context| {
            configure_fonts(&creation_context.egui_ctx);
            Ok(Box::new(WindowInfoApp::new(&creation_context.egui_ctx)))
        }),
    )
}
