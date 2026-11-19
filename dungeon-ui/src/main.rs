//! Window that draws generated dungeon floors, a port of the Tkinter UI of the
//! Python generator.

mod app;
mod dialog;
mod files;
mod floor_view;
mod geometry;
mod history;
mod legend;
mod palette;
mod reveal;

use clap::Parser;
use eframe::egui;

use crate::app::{DungeonApp, TITLE};

#[derive(Parser, Debug)]
#[command(about = "Shows generated dungeon floors.")]
struct Args {
    /// Seed for the first floor, default is a random seed
    #[arg(short, long)]
    seed: Option<String>,

    /// Id for the first floor, higher floor ids get more rooms
    #[arg(short, long = "floor", default_value_t = 2)]
    floor: u32,
}

fn main() -> eframe::Result {
    let args = Args::parse();
    let seed = args
        .seed
        .filter(|seed| !seed.is_empty())
        .unwrap_or_else(app::random_seed);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(TITLE)
            .with_inner_size([1400.0, 620.0])
            .with_min_inner_size([760.0, 420.0]),
        ..Default::default()
    };
    eframe::run_native(
        TITLE,
        options,
        Box::new(move |cc| Ok(Box::new(DungeonApp::new(&cc.egui_ctx, &seed, args.floor)))),
    )
}
