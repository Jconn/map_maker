// Forbid warnings in release builds:
//#![cfg_attr(not(debug_assertions), deny(warnings))]
#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
use eframe::{egui, epi};
mod widgets;
use std::thread;
use tokio::runtime::Runtime;
use widgets::map_tile;
use std::collections::HashMap;

struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Self { name, age } = self;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                ui.label("Your name: ");
                ui.text_edit_singleline(name);
            });
            ui.add(egui::Slider::new(age, 0..=120).text("age"));
            let tpng: String = "texture.png".to_string();
            ui.add(map_tile::MapTile::load_img(
                "/hdd/rust/projects/map_maker/map_maker/src/terrain.png",
                frame,
            ));
            if ui.button("Click each year").clicked() {
                *age += 1;
            }
            ui.label(format!("Hello '{}', age {}", name, age));
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }
}
fn tokio_runtime_thread() {
    let mut rt = Runtime::new().unwrap();
    let handle = rt.spawn(async {
        let resp = reqwest::get("https://stamen-tiles.a.ssl.fastly.net/terrain/2/1/3.png").await;
        println!("response: {:#?}", resp);
    });
    rt.block_on(handle);
}
fn main() {
    let tokio_thread_handle = thread::spawn(tokio_runtime_thread);
    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(MyApp::default()), options);
    tokio_thread_handle.join().unwrap();
}
