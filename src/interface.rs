
use eframe::egui;
use crate::app::App;
use crate::{data::get_tiling_type, tiling::IsohedralTiling};
use rand::{thread_rng, Rng};
use egui_colors::utils;


fn draw_isohedrals(app: &mut App, ctx: &egui::Context) {
    let tokens = app.colorix.animator.animated_tokens;
    let rect = ctx.screen_rect();
    let layer_id = egui::LayerId::background();
    let painter = egui::Painter::new(ctx.clone(), layer_id, rect);
    let colors = [tokens.active_ui_element_background(), tokens.solid_backgrounds(), tokens.hovered_ui_element_border()];
    let stroke = egui::Stroke::new(3., tokens.low_contrast_text());

    painter.extend(app.tiling.fill_region(-2., -2., 20., 20.).iter().map(|tile| {
        let c = colors[app.tiling.colour(tile.t1, tile.t2, tile.aspect)];
        let mut points = vec![];

        app.tiling.shapes().into_iter().for_each(|e| {
            let edge = &app.edges_shapes[e.id()];
            let transform = tile.transform * e.transform();
            let p1 = transform.transform_point2(edge[0]);
            let p2 = transform.transform_point2(edge[1]);
            let point1 = egui::pos2(p1.x as f32 * 100., p1.y as f32 * 100.);
            let point2 = egui::pos2(p2.x as f32 * 100., p2.y as f32 * 100.);

            if points.len() < 1 {
                points.push(point1)
            }
            if e.reversed() {
                points.push(point1);
            }
            else {
                points.push(point2);
            }
        });
        egui::Shape::convex_polygon(points, c, stroke)
    }
    ))
}

pub fn draw_interface(app: &mut App, ctx: &egui::Context) {
    ctx.style_mut(|style| {
        style.visuals.panel_fill = app.colorix.animator.animated_tokens.subtle_background(); 
    });
    egui::Window::new("Isohedrals").show(ctx, |ui| {
        ui.horizontal(|ui| {
            app.colorix.light_dark_toggle_button(ui, 30.);
            ui.add_space(10.);
            app.colorix.themes_dropdown(ui, None, false);
        });
        ui.vertical_centered(|ui| {
            let type_nr = app.tile_type_num;
            ui.add_space(5.);
            if ui.add(egui::Slider::new(&mut app.tile_type_num, 0..=80).text(format!("type: {}", get_tiling_type(type_nr)))).changed() {
                app.tiling = IsohedralTiling::new(get_tiling_type(app.tile_type_num));
                app.set_default_edges();
                app.set_default_params();
            };
            for i in 0..app.tiling.num_params {
                ui.add_space(5.);
                if ui.add(egui::Slider::new(&mut app.params[i], 0.0..=1.).text(format!("v{}", i))).changed() {
                    app.tiling.set_parameters(&app.params);
                };
            };
            let mut rng = thread_rng();
            if ui.button("Random theme").clicked() {
                app.set_params = true;
                let rand_theme = rng.gen_range(0..8);
                app.colorix.update_theme(ctx, utils::THEMES[rand_theme]) 
            }  
            if app.set_params {
                let (r, g, b, _) = app.colorix.animator.tokenshifts[2].to_tuple();
                let (r2, g2, b2, _) = app.colorix.animator.tokenshifts[1].to_tuple();
                let params = [r as f32/ 255., g as f32/ 255., b as f32/ 255., r2 as f32/ 255., g2 as f32/ 255., b2 as f32/ 255.];
                if app.tiling.num_params != 0 {
                    let rand_param = rng.gen_range(0..app.tiling.num_params);
                    app.params[rand_param] = params[rand_param];
                    app.tiling.set_parameters(&app.params); 
                }
                if app.colorix.animator.progress == 1. {
                    app.set_params = false
                }
            }         
        })
    });
    draw_isohedrals(app, ctx);
}