mod utils;

use eframe::egui::{CentralPanel, Context};
use eframe::{App, CreationContext, Frame};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn start() {
    utils::set_panic_hook();
    eframe::start_web("view", Box::new(|_ctx: &CreationContext| Box::new(Ui {}))).unwrap();
}

pub struct Ui {}

impl App for Ui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
        });
    }
}
