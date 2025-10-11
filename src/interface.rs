
use eframe::egui::{self, Color32, Mesh, Pos2};
use crate::app::App;
use crate::{data::get_tiling_type, tiling::IsohedralTiling};
use rand::{thread_rng, Rng};
use egui_colors::utils;
use lyon::math::Point;
use lyon::path::Path;
use lyon::tessellation::{FillOptions, FillTessellator, StrokeOptions, StrokeTessellator, VertexBuffers};
use lyon::tessellation::geometry_builder::simple_builder;


fn create_path(points: &[Point]) -> Path {
    let mut builder = Path::builder();
    builder.begin(points[0]);

    for &p in &points[1..] {
        builder.line_to(p);
    }
    builder.close();
    builder.build()
}


fn tesselate_polygon(points: &[Point]) -> VertexBuffers<Point, u16> {
    let path = create_path(points);

    let mut tessellator = FillTessellator::new();
    let mut geometry = VertexBuffers::new();

    let mut vertex_builder = simple_builder(&mut geometry);

    tessellator.tessellate_path(
        &path,
        &FillOptions::default(),
        &mut vertex_builder
    ).unwrap();

    geometry
}

fn tesselate_stroke(points: &[Point], width: f32) -> VertexBuffers<Point, u16> {
    let path = create_path(points);

    let mut tessellator = StrokeTessellator::new();
    let mut geometry = VertexBuffers::new();

    let mut vertex_builder = simple_builder(&mut geometry);

    tessellator.tessellate_path(
        &path,
        &StrokeOptions::default().with_line_width(width),
        &mut vertex_builder
    ).unwrap();

    geometry
}

fn to_egui_mesh(geometry: VertexBuffers<Point, u16>, color: Color32) -> Mesh {
    let mut mesh = Mesh::default();

    for v in geometry.vertices {
        mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(v.x, v.y), uv: egui::epaint::WHITE_UV, color });
    }
    mesh.indices = geometry.indices.into_iter().map(|i| i as u32).collect();

    mesh
}

fn draw_isohedrals(app: &mut App, ctx: &egui::Context) {
    let tokens = app.colorix.animator.animated_tokens;
    let rect = ctx.content_rect();
    let layer_id = egui::LayerId::background();
    let painter = egui::Painter::new(ctx.clone(), layer_id, rect);
    let colors = [tokens.active_ui_element_background(), tokens.solid_backgrounds(), tokens.hovered_ui_element_border()];

    painter.extend(app.tiling.fill_region(-5., -5., 20., 20.).iter().flat_map(|tile| {
        let c = colors[app.tiling.colour(tile.t1, tile.t2, tile.aspect)];
        let mut points = vec![];

        app.tiling.shapes().for_each(|e| {
            let edge = &app.edges_shapes[e.id()];
            let transform = tile.transform * e.transform();
            let p1 = transform.transform_point2(edge[0]);
            let p2 = transform.transform_point2(edge[1]);
            let point1 = Point::new(p1.x * 100., p1.y * 100.);
            let point2 = Point::new(p2.x * 100., p2.y * 100.);

            if points.is_empty() {
                points.push(point1)
            }
            if e.reversed() {
                points.push(point1);
            }
            else {
                points.push(point2);
            }
        });
        let stroke = tesselate_stroke(&points, 5.);
        let geometry = tesselate_polygon(&points);
        let stroke_mesh = to_egui_mesh(stroke, tokens.low_contrast_text());
        let mesh = to_egui_mesh(geometry, c);
        
        vec![
        egui::Shape::mesh(mesh),
        egui::Shape::mesh(stroke_mesh)]
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