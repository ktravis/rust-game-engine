use std::collections::HashMap;

use glam::{vec2, Vec2};
use image::{Pixel, Rgba, RgbaImage};
use msdfgen::{
    Bitmap, Bound, FillRule, FontExt, Framing, MsdfGeneratorConfig, Projection, Range, Vector2,
};

use crate::{
    atlas::{Atlas, AtlasBuilder},
    geom::Rect,
};
use ttf_parser::Face;

#[derive(Debug, Copy, Clone)]
pub struct GlyphMetrics {
    pub bounds: Rect,
    tex_pos_offset: Vec2,
    tex_scale: Vec2,
    pub bounds_raw: Bound<f64>,
    pub framing: Framing<f64>,
}

#[derive(Debug, Copy, Clone)]
pub struct GlyphData {
    pub metrics: GlyphMetrics,
    pub subtexture: Rect,
}

impl AsRef<GlyphMetrics> for GlyphData {
    fn as_ref(&self) -> &GlyphMetrics {
        &self.metrics
    }
}

impl std::ops::Deref for GlyphData {
    type Target = GlyphMetrics;

    fn deref(&self) -> &Self::Target {
        &self.metrics
    }
}

pub struct FontAtlas {
    atlas: Atlas,
    glyph_data: HashMap<char, GlyphData>,
    pub line_height: i16,
    pub pixels_per_em: u32,
    pub pixel_scale: f32,
    pub units_per_em: u16,
}

pub struct FontAtlasConfig<T = std::ops::Range<char>> {
    pixels_per_em: u32,
    chars: T,
}

impl Default for FontAtlasConfig {
    fn default() -> Self {
        Self {
            pixels_per_em: 24,
            chars: '\x21'..'\x7e',
        }
    }
}

impl FontAtlas {
    pub fn new<T: Iterator<Item = char>>(
        face: Face,
        config: FontAtlasConfig<T>,
    ) -> anyhow::Result<Self> {
        let mut atlas_builder = AtlasBuilder::default();
        let fill_rule = FillRule::default();
        let mut glyph_data = HashMap::default();
        let mut glyphs = vec![];
        let units_per_em = face.units_per_em();
        let pixels_per_unit = config.pixels_per_em as f32 / units_per_em as f32;
        let msdf_config = MsdfGeneratorConfig::default();
        let resolution = config.pixels_per_em;
        let mut map = Bitmap::new(resolution, resolution);
        let mut original_size = None;
        for c in config.chars {
            assert!(!c.is_whitespace());
            let glyph_id = face.glyph_index(dbg!(c)).unwrap();
            let mut shape = face.glyph_shape(glyph_id).unwrap();

            let mut shape_bounds = shape.get_bound();
            let framing = shape_bounds
                .autoframe(
                    resolution,
                    resolution,
                    Range::Px(2. * (resolution / 16) as f64),
                    None,
                )
                .unwrap();
            let origin = *original_size.get_or_insert(framing.range);

            shape.edge_coloring_simple(3., 0);
            shape.generate_mtsdf(&mut map, &framing, msdf_config);
            shape.correct_sign(&mut map, &framing, fill_rule);
            shape.correct_msdf_error(&mut map, &framing, &msdf_config);

            std::mem::swap(&mut shape_bounds.bottom, &mut shape_bounds.top);
            map.flip_y();

            let bitmap: Bitmap<msdfgen::Rgba<u8>> = map.convert();

            let (top, left, bottom, right) = (
                (shape_bounds.top + framing.translate.y) * framing.scale.y,
                (shape_bounds.left + framing.translate.x) * framing.scale.x,
                (shape_bounds.bottom + framing.translate.y) * framing.scale.y,
                (shape_bounds.right + framing.translate.x) * framing.scale.x,
            );

            let scale = framing.range / origin;
            let bounds_bottom =
                (-shape_bounds.top / resolution as f64 * framing.scale.y * scale) as f32;
            let bounds_top =
                (-shape_bounds.bottom / resolution as f64 * framing.scale.y * scale) as f32;
            let bounds_left =
                (shape_bounds.left / resolution as f64 * framing.scale.x * scale) as f32;
            let bounds_right =
                (shape_bounds.right / resolution as f64 * framing.scale.x * scale) as f32;

            glyphs.push((
                c,
                GlyphMetrics {
                    bounds: Rect {
                        pos: vec2(bounds_left, bounds_top) * resolution as f32,
                        dim: vec2(bounds_right - bounds_left, bounds_bottom - bounds_top)
                            * resolution as f32,
                    },
                    tex_pos_offset: vec2(left as f32, top as f32),
                    tex_scale: vec2(
                        (right - left) as f32 / resolution as f32,
                        (bottom - top) as f32 / resolution as f32,
                    ),
                    bounds_raw: shape_bounds,
                    framing,
                },
            ));

            let mut img = RgbaImage::new(resolution, resolution);
            // TODO: it would be nice if we didn't have to copy the image here
            bitmap
                .pixels()
                .iter()
                .zip(img.pixels_mut())
                .for_each(|(src, dst)| *dst = Rgba::from_channels(src.r, src.g, src.b, src.a));
            atlas_builder.add(&img)?;
        }
        let atlas = atlas_builder.build();
        for (i, (c, metrics)) in glyphs.into_iter().enumerate() {
            let mut subtexture = atlas.entry_rect(i);
            subtexture.pos.x += metrics.tex_pos_offset.x / atlas.size().x as f32;
            subtexture.pos.y += metrics.tex_pos_offset.y / atlas.size().y as f32;
            subtexture.dim.x *= metrics.tex_scale.x;
            subtexture.dim.y *= metrics.tex_scale.y;
            glyph_data.insert(
                c,
                GlyphData {
                    metrics,
                    subtexture,
                },
            );
        }
        // for subtable in face.tables().kern.unwrap().subtables {
        //     let subtable = &subtable;
        //     if !subtable.horizontal {
        //         continue;
        //     }
        //     match &subtable.format {
        //         ttf_parser::kern::Format::Format0(f) => {
        //             println!(
        //                 "kern cd: {:?}",
        //                 f.glyphs_kerning(
        //                     face.glyph_index('c').unwrap(),
        //                     face.glyph_index('d').unwrap()
        //                 )
        //             );
        //         }
        //         ttf_parser::kern::Format::Format1(f) => todo!(),
        //         ttf_parser::kern::Format::Format2(f) => todo!(),
        //         ttf_parser::kern::Format::Format3(f) => todo!(),
        //     }
        // }

        Ok(FontAtlas {
            atlas,
            glyph_data,
            line_height: face.line_gap() + face.height(),
            pixels_per_em: config.pixels_per_em,
            pixel_scale: pixels_per_unit,
            units_per_em,
        })
    }

    pub fn image(&self) -> &RgbaImage {
        self.atlas.image()
    }

    pub fn glyph_data(&self, glyph: char) -> GlyphData {
        *self
            .glyph_data
            .get(&glyph)
            .expect(&format!("glyph not found in atlas: {}", glyph))
    }
}

// pub fn make_font_atlas(path: &str) -> anyhow::Result<Atlas> {
//     let b = std::fs::read(path)?;
//     let mut atlas_builder = AtlasBuilder::default();
//     let face = Face::parse(&b, 0).unwrap();
//     let width = 32;
//     let height = 32;
//     let fill_rule = FillRule::default();
//     let mut bitmap = Bitmap::new(width, height);
//     for c in '\x21'..'\x7e' {
//         let glyph_index = face.glyph_index(c).unwrap();
//         let mut shape = face.glyph_shape(glyph_index).unwrap();

//         let bound = shape.get_bound();
//         let framing = bound.autoframe(width, height, Range::Px(4.), None).unwrap();

//         shape.edge_coloring_simple(3., 0);

//         let config = MsdfGeneratorConfig::default();

//         shape.generate_msdf(&mut bitmap, framing, config);

//         shape.correct_sign(&mut bitmap, &framing, fill_rule);
//         shape.correct_msdf_error(&mut bitmap, &framing, &config);

//         bitmap.flip_y();

//         let mut img = RgbaImage::new(width, height);
//         bitmap
//             .pixels()
//             .iter()
//             .zip(img.pixels_mut())
//             .for_each(|(src, dst)| {
//                 *dst = Rgba::from_channels(
//                     (src.r * 255.) as u8,
//                     (src.g * 255.) as u8,
//                     (src.b * 255.) as u8,
//                     255,
//                 )
//             });
//         atlas_builder.add(&img)?;
//     }
//     Ok(atlas_builder.build())
// }

// #[cfg(tests)]
// mod tests {
//     use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
//     use image::{DynamicImage, Pixel, RgbImage, Rgba, RgbaImage};

//     use crate::atlas::AtlasBuilder;

//     use super::*;

//     #[test]
//     fn test_glyph_metrics() {
//         let b = std::fs::read("/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf").unwrap();
//         let face = Face::parse(&b, 0).unwrap();
//         // let font_atlas = FontAtlas::new(face, Default::default()).unwrap();
//         let fill_rule = FillRule::default();
//         let units_per_em = face.units_per_em();
//         let pixels_per_em = 24;
//         let pixels_per_unit = pixels_per_em as f32 / units_per_em as f32;
//         let msdf_config = MsdfGeneratorConfig::default();
//         let chars = "abcdefghijkl|.".chars();
//         // let mut bitmap = Bitmap::new(500, 100);
//         for c in chars {
//             let glyph_id = face.glyph_index(c).unwrap();
//             let mut shape = face.glyph_shape(glyph_id).unwrap();
//             shape.edge_coloring_simple(3., 0);
//             let unit_bounds = shape.get_bound();
//             let bounds = Rect::new(
//                 unit_bounds.left as f32,     // * pixels_per_unit,
//                 unit_bounds.top as f32,      // * pixels_per_unit,
//                 unit_bounds.width() as f32,  // * pixels_per_unit,
//                 unit_bounds.height() as f32, // * pixels_per_unit,
//             );
//             let width = (unit_bounds.width() as f32 * pixels_per_unit).round() as u32 + 8;
//             let height = (unit_bounds.height() as f32 * pixels_per_unit).round() as u32 + 8;
//             let width = width.max(16);
//             let height = height.max(16);
//             let framing = unit_bounds
//                 .autoframe(
//                     width,
//                     height,
//                     Range::Px(2.0),
//                     // Some(msdfgen::Vector2 { x: 1., y: 1. }),
//                     None,
//                 )
//                 .unwrap();
//             println!("####### ( {} )", c);
//             println!("width: {}", width);
//             println!("height: {}", height);
//             println!("framing.translate: {:?}", framing.translate);
//             println!("framing.scale: {:?}", framing.scale);
//             dbg!(framing.project(Vector2::new(unit_bounds.left, unit_bounds.bottom)));
//             dbg!(framing.project(Vector2::new(unit_bounds.right, unit_bounds.top)));
//             dbg!(face.glyph_hor_side_bearing(glyph_id).unwrap());
//             dbg!(face.glyph_hor_side_bearing(glyph_id).unwrap() as f32 * pixels_per_unit as f32);
//             dbg!(face.ascender() as f32 * pixels_per_unit);
//             dbg!(face.descender() as f32 * pixels_per_unit);
//             let scale_y = pixels_per_em as f32 / (face.ascender() - face.descender()) as f32;
//             let scale_x = scale_y / pixels_per_em as f32;
//             dbg!(vec2(scale_x, scale_y));
//             dbg!(face.glyph_hor_side_bearing(glyph_id).unwrap() as f32 * scale_x);
//             let bb = face.glyph_bounding_box(glyph_id).unwrap();
//             dbg!(bb);
//             dbg!((bb.x_max - bb.x_min) as f64 * framing.projection.scale.x);
//             dbg!((bb.y_max - bb.y_min) as f64 * framing.projection.scale.y);
//             // dbg!(unit_bounds.left * scale_x as f64);
//             // dbg!(unit_bounds.right * scale_x as f64);
//             // dbg!(unit_bounds.width() * scale_x as f64);
//             // dbg!(unit_bounds.top * scale_y as f64);
//             // dbg!(unit_bounds.bottom * scale_y as f64);
//             // dbg!(unit_bounds.height() * scale_y as f64);

//             // let projection = dbg!(framing.projection);
//         }
//     }

//     #[test]
//     fn test_fontdue() {
//         use fontdue::Font;
//         let font = include_bytes!("/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf") as &[u8];
//         let face = Face::parse(font, 0).unwrap();
//         // Parse it into the font type.
//         let roboto_regular = Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
//         // The list of fonts that will be used during layout.
//         let fonts = &[roboto_regular];
//         // Create a layout context. Laying out text needs some heap allocations; reusing this context
//         // reduces the need to reallocate space. We inform layout of which way the Y axis points here.
//         let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
//         // By default, layout is initialized with the default layout settings. This call is redundant, but
//         // demonstrates setting the value with your custom settings.
//         layout.reset(&LayoutSettings {
//             ..LayoutSettings::default()
//         });
//         // The text that will be laid out, its size, and the index of the font in the font list to use for
//         // that section of text.
//         layout.append(fonts, &TextStyle::new("abcdefghijkl|.", 24.0, 0));

//         let bw = layout
//             .glyphs()
//             .last()
//             .and_then(|g| Some(g.x + g.width as f32))
//             .unwrap()
//             + 100.;
//         let bh = layout.height() as u32 + 10;

//         let units_per_em = face.units_per_em();
//         let pixels_per_em = 24;
//         let pixels_per_unit = pixels_per_em as f32 / units_per_em as f32;
//         let msdf_config = MsdfGeneratorConfig::default();
//         let chars = "abcdefghijkl|.".chars();
//         let mut img = RgbaImage::new(bw as u32, bh);
//         let font_atlas = FontAtlas::new(
//             face.clone(),
//             FontAtlasConfig {
//                 pixels_per_em: 24,
//                 ..Default::default()
//             },
//         )
//         .unwrap();

//         // Prints the layout for "Hello world!"
//         for g in layout.glyphs() {
//             println!("fontdue glyph: {:#?}", g);

//             let glyph_id = face.glyph_index(g.parent).unwrap();
//             let mut shape = face.glyph_shape(glyph_id).unwrap();
//             shape.edge_coloring_simple(3., 0);
//             let unit_bounds = shape.get_bound();
//             let bounds = Rect::new(
//                 unit_bounds.left as f32 * pixels_per_unit,
//                 unit_bounds.top as f32 * pixels_per_unit,
//                 unit_bounds.width() as f32 * pixels_per_unit,
//                 unit_bounds.height() as f32 * pixels_per_unit,
//             );
//             let width = (unit_bounds.width() as f32 * pixels_per_unit).round() as u32 + 8;
//             let height = (unit_bounds.height() as f32 * pixels_per_unit).round() as u32 + 8;
//             let width = width.max(16);
//             let height = height.max(16);
//             let framing = unit_bounds
//                 .autoframe(
//                     width,
//                     height,
//                     Range::Px(2.0),
//                     // Some(msdfgen::Vector2 { x: 1., y: 1. }),
//                     None,
//                 )
//                 .unwrap();
//             println!("framing: {:#?}", framing);
//             println!("bounds: {:#?}", bounds);
//             println!("unit_bounds: {:#?}", unit_bounds);
//             println!("---------------");
//             let mut bitmap = Bitmap::new(width, height);
//             shape.generate_mtsdf(&mut bitmap, framing, &msdf_config);

//             let fill_rule = FillRule::default();
//             shape.correct_sign(&mut bitmap, &framing, fill_rule);
//             shape.correct_msdf_error(&mut bitmap, &framing, &msdf_config);

//             bitmap.flip_y();
//             for (i, px) in bitmap.pixels().iter().enumerate() {
//                 let x = i as u32 % bitmap.width();
//                 let y = i as u32 / bitmap.height();
//                 img.get_pixel_mut(g.x as u32 + x, g.y as u32 + y).0 = [
//                     (px.r * 255.) as u8,
//                     (px.g * 255.) as u8,
//                     (px.b * 255.) as u8,
//                     (px.a * 255.) as u8,
//                 ];
//             }
//         }

//         img.save("blah.png").unwrap();
//     }

//     #[test]
//     fn test_w() {
//         let b = std::fs::read("/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf").unwrap();
//         let face = Face::parse(&b, 0).unwrap();
//         // let font_atlas = FontAtlas::new(face, Default::default()).unwrap();
//         let resolution = 24;
//         let mut map = Bitmap::new(resolution, resolution);

//         let fill_rule = FillRule::default();
//         let units_per_em = face.units_per_em();
//         let pixels_per_em = resolution;
//         let pixels_per_unit = pixels_per_em as f32 / units_per_em as f32;
//         let msdf_config = MsdfGeneratorConfig::default();
//         let asc = face.ascender() as f32;
//         let desc = face.descender() as f32;
//         let toppest = asc / (asc - desc);
//         let lowest = desc / (asc - desc);

//         let glyph_id = face.glyph_index('a').unwrap();
//         let mut shape = face.glyph_shape(glyph_id).unwrap();

//         let mut shape_bounds = shape.get_bound();
//         let framing = shape_bounds
//             .autoframe(
//                 resolution,
//                 resolution,
//                 Range::Px(2.0 * (resolution / 16) as f64),
//                 None,
//             )
//             .unwrap();
//         let origin = framing.range;

//         shape.edge_coloring_simple(3., 0);
//         shape.generate_msdf(&mut map, &framing, msdf_config);

//         std::mem::swap(&mut shape_bounds.bottom, &mut shape_bounds.top);
//         map.flip_y();

//         let mapu8: Bitmap<msdfgen::Rgb<u8>> = map.convert();

//         let (top, left, bottom, right) = (
//             (shape_bounds.top + framing.translate.y) * framing.scale.y,
//             (shape_bounds.left + framing.translate.x) * framing.scale.x,
//             (shape_bounds.bottom + framing.translate.y) * framing.scale.y,
//             (shape_bounds.right + framing.translate.x) * framing.scale.x,
//         );

//         let scale = framing.range / origin;
//         let bounds_bottom =
//             (-shape_bounds.top / resolution as f64 * framing.scale.y * scale) as f32;
//         let bounds_top =
//             (-shape_bounds.bottom / resolution as f64 * framing.scale.y * scale) as f32;
//         let bounds_left = (shape_bounds.left / resolution as f64 * framing.scale.x * scale) as f32;
//         let bounds_right =
//             (shape_bounds.right / resolution as f64 * framing.scale.x * scale) as f32;

//         dbg!(shape_bounds);
//         dbg!(framing.scale);
//         dbg!(origin);
//         dbg!(scale);
//         dbg!(bounds_bottom);
//         dbg!(bounds_top);
//         dbg!(bounds_left);
//         dbg!(bounds_right);
//         dbg!(top);
//         dbg!(left);
//         dbg!(bottom);
//         dbg!(right);
//         dbg!((right - left) as f32);
//         dbg!((bottom - top) as f32);

//         let mut f = std::fs::File::create("w.png").unwrap();
//         mapu8.write_png(&mut f).unwrap();

//         // let bounds = Rect::new(
//         //     unit_bounds.left as f32,     // * pixels_per_unit,
//         //     unit_bounds.top as f32,      // * pixels_per_unit,
//         //     unit_bounds.width() as f32,  // * pixels_per_unit,
//         //     unit_bounds.height() as f32, // * pixels_per_unit,
//         // );
//         // let width = (unit_bounds.width() as f32 * pixels_per_unit).round() as u32 + 8;
//         // let height = (unit_bounds.height() as f32 * pixels_per_unit).round() as u32 + 8;
//         // let width = width.max(16);
//         // let height = height.max(16);
//         // let framing = unit_bounds
//         //     .autoframe(
//         //         width,
//         //         height,
//         //         Range::Px(2.0),
//         //         // Some(msdfgen::Vector2 { x: 1., y: 1. }),
//         //         None,
//         //     )
//         //     .unwrap();
//         // let projection = dbg!(framing.projection);
//         // println!("width: {}", width);
//         // println!("height: {}", height);
//         // println!("framing.translate: {:?}", framing.translate);
//         // println!("framing.scale: {:?}", framing.scale);
//         // dbg!(framing.project(Vector2::new(unit_bounds.left, unit_bounds.bottom)));
//         // dbg!(framing.project(Vector2::new(unit_bounds.right, unit_bounds.top)));
//         // dbg!(face.glyph_hor_side_bearing(glyph_id).unwrap());
//         // dbg!(face.glyph_hor_side_bearing(glyph_id).unwrap() as f32 * pixels_per_unit as f32);
//         // dbg!(face.ascender() as f32 * pixels_per_unit);
//         // dbg!(face.descender() as f32 * pixels_per_unit);
//         // let scale_y = pixels_per_em as f32 / (face.ascender() - face.descender()) as f32;
//         // let scale_x = scale_y / pixels_per_em as f32;
//         // dbg!(vec2(scale_x, scale_y));
//         // dbg!(face.glyph_hor_side_bearing(glyph_id).unwrap() as f32 * scale_x);
//         // let bb = face.glyph_bounding_box(glyph_id).unwrap();
//         // dbg!(bb);
//         // dbg!((bb.x_max - bb.x_min) as f64 / framing.projection.scale.x);
//         // dbg!((bb.y_max - bb.y_min) as f64 / framing.projection.scale.y);
//     }
// }
