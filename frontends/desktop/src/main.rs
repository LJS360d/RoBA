// main.rs
use clap::Parser;
use eframe::egui;
use egui::IconData;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

// Define command-line arguments using clap's derive feature.
#[derive(Parser, Debug)]
#[command(version, about = "A Game Boy Advance emulator.", long_about = None)]
struct Args {
    /// Path to a GBA ROM file to open immediately.
    #[arg(name = "ROM_PATH")]
    rom_path: Option<PathBuf>,
}

// Configuration struct for serialization.
#[derive(Serialize, Deserialize, Default)]
struct Config {
    recent_files: Vec<PathBuf>,
}

// Function to get the configuration directory.
fn config_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "RoBA", "RoBA")
        .map(|dirs| dirs.config_dir().to_path_buf())
}

// Function to load the configuration from a file.
fn load_config() -> Config {
    let Some(mut path) = config_dir() else {
        return Config::default();
    };
    path.push("config.toml");
    let Ok(config_str) = fs::read_to_string(&path) else {
        return Config::default();
    };
    toml::from_str(&config_str).unwrap_or_default()
}

// Function to save the configuration to a file.
fn save_config(config: &Config) -> io::Result<()> {
    if let Some(mut path) = config_dir() {
        fs::create_dir_all(&path)?;
        path.push("config.toml");
        let config_str =
            toml::to_string(config).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(&path, config_str)?;
    }
    Ok(())
}

// The main application state, handling whether we're in the file selection view or emulation view.
enum AppState {
    FileSelection,
    Emulation(PathBuf),
}

// The main application struct.
struct GbaApp {
    state: AppState,
    recent_files: Vec<PathBuf>,
    core: core::Emulator,
    texture: Option<egui::TextureHandle>,
}

impl GbaApp {
    // A constructor for the application, handling the initial state based on command-line arguments.
    fn new(rom_path: Option<PathBuf>) -> Self {
        let config = load_config();
        let core = core::Emulator::new();

        if let Some(path) = rom_path {
            let mut recent_files = config.recent_files;
            // Add the new path to the recent files list and remove duplicates.
            Self::add_to_recent(&mut recent_files, path.clone());
            Self {
                state: AppState::Emulation(path),
                recent_files,
                core,
                texture: None,
            }
        } else {
            Self {
                state: AppState::FileSelection,
                recent_files: config.recent_files,
                core,
                texture: None,
            }
        }
    }

    // Helper function to add a path to the recent files list and manage its length.
    fn add_to_recent(recent: &mut Vec<PathBuf>, path: PathBuf) {
        // Remove the path if it already exists to avoid duplicates.
        if let Some(index) = recent.iter().position(|p| p == &path) {
            recent.remove(index);
        }
        recent.insert(0, path);

        // Keep the list to a reasonable size (e.g., 10 entries).
        recent.truncate(10);
    }

    // Function to handle the "Open ROM" action.
    fn open_rom(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open GBA ROM")
            .add_filter("Game Boy Advance ROM", &["gba"])
            .pick_file()
        {
            Self::add_to_recent(&mut self.recent_files, path.clone());
            self.state = AppState::Emulation(path);
        }
    }
}

// Implement the eframe::App trait for our GbaApp.
impl eframe::App for GbaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Draw the menu bar.
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open ROM...").clicked() {
                        self.open_rom();
                        ui.close_menu();
                    }
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                // Add more menus here as needed.
                ui.menu_button("Window", |ui| {
                    let _ = ui.button("Settings");
                });
            });
        });

        // Draw the central panel for the main content.
        egui::CentralPanel::default().show(ctx, |ui| {
            // Based on the current application state, draw the appropriate view.
            match &self.state {
                AppState::FileSelection => {
                    ui.heading("Recently Opened GBA ROMs");
                    ui.separator();

                    if self.recent_files.is_empty() {
                        ui.label(
                            "No recent files found. Use 'File -> Open ROM...' to get started.",
                        );
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for file in &self.recent_files {
                                if ui.button(file.display().to_string()).clicked() {
                                    self.state = AppState::Emulation(file.clone());
                                }
                            }
                        });
                    }
                }
                AppState::Emulation(rom_path) => {
                    ui.heading("Emulating GBA ROM");
                    ui.label(format!("Now emulating: {}", rom_path.display()));
                    ui.separator();

                    // Ensure ROM loaded once
                    if self.texture.is_none() {
                        self.core.load_rom(rom_path);
                    }

                    // Step one frame
                    self.core.run_frame();

                    // Upload texture and draw
                    let rgba = self.core.framebuffer_rgba();
                    let size = [core::video::GBA_SCREEN_W as usize, core::video::GBA_SCREEN_H as usize];
                    let image = egui::ColorImage::from_rgba_unmultiplied(size, rgba);
                    let tex = self.texture.get_or_insert_with(|| {
                        ui.ctx().load_texture(
                            "framebuffer",
                            image.clone(),
                            egui::TextureOptions::NEAREST,
                        )
                    });
                    tex.set(image, egui::TextureOptions::NEAREST);

                    let scale = 2.0;
                    let desired = egui::Vec2::new(
                        core::video::GBA_SCREEN_W as f32 * scale,
                        core::video::GBA_SCREEN_H as f32 * scale,
                    );
                    let sized = egui::load::SizedTexture { id: tex.id(), size: tex.size_vec2() };
                    ui.add(egui::Image::new(sized).fit_to_exact_size(desired));
                }
            }
        });
    }

    // Save the recent files list when the application is about to quit.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let config = Config {
            recent_files: self.recent_files.clone(),
        };
        if let Err(e) = save_config(&config) {
            eprintln!("Failed to save config: {}", e);
        }
    }
}

// The main entry point of the application.
fn main() -> eframe::Result<()> {
    // Parse command-line arguments.
    let args = Args::parse();
    let icon = IconData::default(); // from_png_bytes(include_bytes!("../assets/icon.png")).unwrap();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("RoBA - GBA Emulator")
            .with_app_id("com.roba.gba")
            .with_icon(icon)
            ,
        ..Default::default()
    };

    eframe::run_native(
        "RoBA",
        native_options,
        Box::new(|_cc| Ok(Box::new(GbaApp::new(args.rom_path)))),
    )
}
