// #![allow(dead_code)]

use crate::{data::get_tiling_type, interface, tiling::IsohedralTiling, utils::{Vec2, vec2}};
use eframe::egui;
use egui_colors::{utils, Colorix};


#[derive(Default)]
pub struct App {
    pub colorix: Colorix,
    pub params: [f32; 6],
    pub tile_type_num: usize,
    pub tiling: IsohedralTiling,
    pub edges_shapes: Vec<Vec<Vec2>>,
    pub set_params: bool,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.colorix.set_animator(ctx);
        interface::draw_interface(self, ctx);
    }
}

impl App {
    fn new(ctx: &mut egui::Context) -> Self {
        ctx.set_theme(egui::Theme::Light);
        let colorix = Colorix::global(ctx, utils::SEVENTIES).animated().set_time(2.);
        let tile_type_num = 0;
        let tiling = IsohedralTiling::new(get_tiling_type(tile_type_num));

        let mut app = App {
            colorix,
            params: [0.; 6],
            tile_type_num,
            tiling,
            edges_shapes: vec![],
            set_params: false,
        };
        app.set_default_edges();
        app.set_default_params();

        app
    }

    pub fn set_default_edges(&mut self) {
        self.edges_shapes.clear();
        for _ in 0..self.tiling.num_edge_shapes() {
            let mut edge = vec![];
            edge.push(vec2(0.0, 0.0));
            edge.push(vec2(1.0, 0.0));
            self.edges_shapes.push(edge);
        }
    }
    pub fn set_default_params(&mut self) {
        self.params = self.tiling.parameters  
    }
    pub fn _set_params(&mut self, i: usize) {
        self.tiling.parameters[i] = self.params[i]
    }
}

pub fn init() -> Result<(), eframe::Error> {

    eframe::run_native(
        "egui Isohedral",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            Ok(Box::new(App::new(&mut cc.egui_ctx.clone())))
        }),
    )
}