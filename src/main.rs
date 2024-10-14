use noise::{NoiseFn, Perlin};
use rand::Rng;
use eframe::{egui};
use egui::Color32;
use egui::{FontDefinitions, FontFamily};
use std::fs::File;
use std::io::Read;

// Define terrain parameters
struct TerrainConfig {
    width: u32,
    height: u32,
    scale: f64,
    octaves: usize,
    persistence: f64,
    lacunarity: f64,
    pixel_size: u32,
}

struct TerrainApp {
    config: TerrainConfig,
    terrain: egui::ColorImage,
    seed: u32,
    texture_handle: Option<egui::TextureHandle>,
}

impl eframe::App for TerrainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut regenerate = false;

        // Set the background color
        let bg_color = Color32::from_rgb(218, 204, 158); // Light brown
        ctx.set_visuals(egui::Visuals {
            window_fill: bg_color,
            panel_fill: bg_color,
            ..Default::default()
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Terrain Generator");
            ui.separator();

            regenerate |= ui.add(egui::Slider::new(&mut self.config.scale, 1.0..=100.0).text("Scale")).changed();
            regenerate |= ui.add(egui::Slider::new(&mut self.config.octaves, 1..=8).text("Octaves")).changed();
            regenerate |= ui.add(egui::Slider::new(&mut self.config.persistence, 0.0..=1.0).text("Persistence")).changed();
            regenerate |= ui.add(egui::Slider::new(&mut self.config.lacunarity, 1.0..=4.0).text("Lacunarity")).changed();
            regenerate |= ui.add(egui::Slider::new(&mut self.config.pixel_size, 1..=16).text("Pixel Size")).changed();

            if ui.button("New Seed").clicked() {
                self.seed = rand::thread_rng().gen();
                regenerate = true;
            }

            if let Some(texture_handle) = self.texture_handle.as_ref() {
                ui.image(texture_handle, texture_handle.size_vec2());
            }
        });

        if regenerate {
            self.regenerate_terrain();
            self.update_texture(ctx);
        }
    }
}

impl TerrainApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load custom font
        let mut fonts = FontDefinitions::default();
        
        // Load your custom font file
        let font_data = std::fs::read("src/fonts/OldLondon.ttf").expect("Failed to read font file");
        
        // Add the font to FontDefinitions
        fonts.font_data.insert("my_font".to_owned(), egui::FontData::from_owned(font_data));
        
        // Set the font as the default for various text styles
        fonts.families.get_mut(&FontFamily::Proportional).unwrap()
            .insert(0, "my_font".to_owned());
        fonts.families.get_mut(&FontFamily::Monospace).unwrap()
            .push("my_font".to_owned());

        // Set the font
        cc.egui_ctx.set_fonts(fonts);

        let config = TerrainConfig {
            width: 512,
            height: 512,
            scale: 50.0,
            octaves: 6,
            persistence: 0.5,
            lacunarity: 2.0,
            pixel_size: 1,
        };
        let seed = rand::thread_rng().gen();
        let mut app = Self {
            config,
            terrain: egui::ColorImage::new([256, 256], Color32::BLACK),
            seed,
            texture_handle: None,
        };
        app.regenerate_terrain();
        app.update_texture(&cc.egui_ctx);
        app
    }

    fn regenerate_terrain(&mut self) {
        let perlin = Perlin::new(self.seed);
        let width = self.config.width;
        let height = self.config.height;
        let scale = self.config.scale;
        let octaves = self.config.octaves;
        let persistence = self.config.persistence;
        let lacunarity = self.config.lacunarity;

        let pixels: Vec<Color32> = (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| {
                    let nx = x as f64 / width as f64 - 0.5;
                    let ny = y as f64 / height as f64 - 0.5;

                    let mut noise_value = 0.0;
                    let mut amplitude = 1.0;
                    let mut frequency = 1.0;

                    for _ in 0..octaves {
                        let sample_x = nx * frequency * scale;
                        let sample_y = ny * frequency * scale;
                        noise_value += perlin.get([sample_x, sample_y]) * amplitude;

                        amplitude *= persistence;
                        frequency *= lacunarity;
                    }

                    noise_value = (noise_value + 1.0) / 2.0;
                    Self::get_terrain_color(noise_value)
                })
            })
            .collect();

        self.terrain = egui::ColorImage::from_rgba_unmultiplied(
            [width as _, height as _],
            &pixels.iter().flat_map(|c| c.to_array()).collect::<Vec<u8>>(),
        );
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        self.texture_handle = Some(ctx.load_texture(
            "terrain",
            self.terrain.clone(),
            egui::TextureOptions::NEAREST,
        ));
    }

    fn get_terrain_color(height: f64) -> Color32 {
        let color = match height {
            h if h < 0.3 => [0, 0, 255],    // Deep water
            h if h < 0.4 => [65, 105, 225], // Water
            h if h < 0.5 => [210, 180, 140], // Sand
            h if h < 0.7 => [34, 139, 34],  // Grass
            h if h < 0.8 => [139, 69, 19],  // Mountain
            _ => [255, 255, 255],           // Snow
        };
        Self::quantize_color(color, 1) // Assuming pixel_size is 1 for simplicity
    }

    fn quantize_color(color: [u8; 3], pixel_size: u32) -> Color32 {
        let quantize = |v: u8| {
            let step = 255 / pixel_size;
            ((v as f32 / step as f32).round() * step as f32) as u8
        };

        Color32::from_rgb(
            quantize(color[0]),
            quantize(color[1]),
            quantize(color[2]),
        )
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(530.0, 680.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Terrain Generator",
        options,
        Box::new(|cc| Box::new(TerrainApp::new(cc))),
    )
}
