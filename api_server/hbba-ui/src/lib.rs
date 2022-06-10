mod utils;

use eframe::egui::{CentralPanel, Context, FontData, FontDefinitions, Ui};
use eframe::{App, CreationContext, Frame};
use once_cell::sync::Lazy;
use std::sync::RwLock;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn start() {
    utils::set_panic_hook();
    eframe::start_web(
        "view",
        Box::new(|ctx: &CreationContext| Box::new(Application::new(ctx))),
    )
    .unwrap();
}

pub struct Application {
    load_font: bool,
}

static FONT_DATA: Lazy<RwLock<Option<FontData>>> = Lazy::new(|| {
    fn fetch_font() {
        let req = ehttp::Request::get("res/SourceHanSerifCN-Medium.otf");
        ehttp::fetch(req, |resp| {
            if let Ok(resp) = resp {
                if resp.status == 200 {
                    let mut font = FontData::from_owned(resp.bytes);
                    if let Ok(mut fd) = FONT_DATA.write() {
                        font.tweak.scale = 1.2;
                        *fd = Some(font);
                        return;
                    }
                }
            }
            fetch_font();
        });
    }

    fetch_font();
    RwLock::default()
});

impl Application {
    pub fn new(_ctx: &CreationContext) -> Self {
        Self { load_font: false }
    }

    fn load_font(&mut self, ctx: &Context) -> bool {
        if !self.load_font {
            if let Some(font_data) = FONT_DATA.read().ok().and_then(|f| f.clone()) {
                const FONT_NAME: &str = "思源宋体";
                let mut font_def = FontDefinitions::default();
                font_def.font_data.insert(FONT_NAME.to_string(), font_data);
                for l in font_def.families.values_mut() {
                    l.insert(0, FONT_NAME.to_string());
                }
                ctx.set_fonts(font_def);
                self.load_font = true;
            } else {
                CentralPanel::default().show(ctx, |ui| {
                    ui.centered_and_justified(Ui::spinner);
                });
                ctx.request_repaint();
            }
        }
        self.load_font
    }
}

impl App for Application {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if !self.load_font(ctx) {
            return;
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World! 你好，世界！");
        });
    }
}
