mod app;
pub mod tiling;
mod iterators;
mod utils;
mod data;
mod interface;

fn main() -> Result<(), eframe::Error> {
    app::init()   
}
