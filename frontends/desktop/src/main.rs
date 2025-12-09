use clap::Parser;
use eframe::egui;
use egui::IconData;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "A Game Boy Advance emulator.", long_about = None)]
struct Args {
    #[arg(name = "ROM_PATH")]
    rom_path: Option<PathBuf>,

    #[arg(short, long, name = "BIOS_PATH")]
    bios: Option<PathBuf>,
}

#[derive(Clone)]
struct DisplayLogEntry {
    level: log::Level,
    target: String,
    message: String,
}

impl From<core::log_buffer::LogEntry> for DisplayLogEntry {
    fn from(entry: core::log_buffer::LogEntry) -> Self {
        Self {
            level: entry.level,
            target: entry.target,
            message: entry.message,
        }
    }
}

// Configuration struct for serialization.
#[derive(Serialize, Deserialize, Default)]
struct Config {
    recent_files: Vec<PathBuf>,
    bios_path: Option<PathBuf>,
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
            toml::to_string(config).map_err(io::Error::other)?;
        fs::write(&path, config_str)?;
    }
    Ok(())
}

enum AppState {
    FileSelection,
    Emulation(PathBuf),
}

struct GbaApp {
    state: AppState,
    recent_files: Vec<PathBuf>,
    bios_path: Option<PathBuf>,
    bios_loaded: bool,
    core: core::Emulator,
    texture: Option<egui::TextureHandle>,
    show_debug_panel: bool,
    log_entries: Vec<DisplayLogEntry>,
    auto_scroll_logs: bool,
    log_filter: LogFilter,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LogFilter {
    All,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl GbaApp {
    fn new(rom_path: Option<PathBuf>, cli_bios_path: Option<PathBuf>) -> Self {
        let config = load_config();
        let mut core = core::Emulator::new();

        let bios_path = cli_bios_path
            .or(config.bios_path.clone())
            .or_else(Self::find_default_bios);

        let bios_loaded = if let Some(ref path) = bios_path {
            match core.load_bios(path.as_path()) {
                Ok(()) => true,
                Err(e) => {
                    log::warn!("Failed to load BIOS from {:?}: {}", path, e);
                    false
                }
            }
        } else {
            log::info!("No BIOS path specified, running without BIOS");
            false
        };

        if let Some(path) = rom_path {
            let mut recent_files = config.recent_files;
            Self::add_to_recent(&mut recent_files, path.clone());
            Self {
                state: AppState::Emulation(path),
                recent_files,
                bios_path,
                bios_loaded,
                core,
                texture: None,
                show_debug_panel: cfg!(debug_assertions),
                log_entries: Vec::new(),
                auto_scroll_logs: true,
                log_filter: LogFilter::All,
            }
        } else {
            Self {
                state: AppState::FileSelection,
                recent_files: config.recent_files,
                bios_path,
                bios_loaded,
                core,
                texture: None,
                show_debug_panel: cfg!(debug_assertions),
                log_entries: Vec::new(),
                auto_scroll_logs: true,
                log_filter: LogFilter::All,
            }
        }
    }

    fn find_default_bios() -> Option<PathBuf> {
        log::debug!("Searching for default BIOS...");

        let candidates = [
            PathBuf::from("core/assets/gba_bios.bin"),
            PathBuf::from("assets/gba_bios.bin"),
            PathBuf::from("gba_bios.bin"),
        ];

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let exe_relative = exe_dir.join("gba_bios.bin");
                log::debug!("Checking exe-relative: {:?}", exe_relative);
                if exe_relative.exists() {
                    log::info!("Found BIOS at {:?}", exe_relative);
                    return Some(exe_relative);
                }
            }
        }

        for candidate in &candidates {
            log::debug!("Checking candidate: {:?}", candidate);
            if candidate.exists() {
                log::info!("Found BIOS at {:?}", candidate);
                return Some(candidate.clone());
            }
        }

        if let Some(config_dir) = config_dir() {
            let config_bios = config_dir.join("gba_bios.bin");
            log::debug!("Checking config dir: {:?}", config_bios);
            if config_bios.exists() {
                log::info!("Found BIOS at {:?}", config_bios);
                return Some(config_bios);
            }
        }

        log::warn!("No BIOS found in any default location");
        None
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

    fn poll_logs(&mut self) {
        let new_logs = core::log_buffer::drain_logs();
        for entry in new_logs {
            self.log_entries.push(entry.into());
        }
        const MAX_LOG_ENTRIES: usize = 2000;
        if self.log_entries.len() > MAX_LOG_ENTRIES {
            let excess = self.log_entries.len() - MAX_LOG_ENTRIES;
            self.log_entries.drain(0..excess);
        }
    }

    fn level_color(level: log::Level) -> egui::Color32 {
        match level {
            log::Level::Error => egui::Color32::from_rgb(255, 100, 100),
            log::Level::Warn => egui::Color32::from_rgb(255, 200, 100),
            log::Level::Info => egui::Color32::from_rgb(100, 200, 255),
            log::Level::Debug => egui::Color32::from_rgb(180, 180, 180),
            log::Level::Trace => egui::Color32::from_rgb(120, 120, 120),
        }
    }

    fn filter_matches(&self, level: log::Level) -> bool {
        match self.log_filter {
            LogFilter::All => true,
            LogFilter::Error => level == log::Level::Error,
            LogFilter::Warn => level <= log::Level::Warn,
            LogFilter::Info => level <= log::Level::Info,
            LogFilter::Debug => level <= log::Level::Debug,
            LogFilter::Trace => true,
        }
    }
}

impl eframe::App for GbaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_logs();

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
                ui.menu_button("Window", |ui| {
                    let _ = ui.button("Settings");
                    if ui.checkbox(&mut self.show_debug_panel, "Debug Panel").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });

        if self.show_debug_panel {
            egui::SidePanel::right("debug_panel")
                .resizable(true)
                .min_width(250.0)
                .default_width(350.0)
                .max_width(500.0)
                .show(ctx, |ui| {
                    ui.heading("Debug Log");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Filter:");
                        ui.selectable_value(&mut self.log_filter, LogFilter::All, "All");
                        ui.selectable_value(&mut self.log_filter, LogFilter::Error, "Error");
                        ui.selectable_value(&mut self.log_filter, LogFilter::Warn, "Warn");
                        ui.selectable_value(&mut self.log_filter, LogFilter::Info, "Info");
                        ui.selectable_value(&mut self.log_filter, LogFilter::Debug, "Debug");
                        ui.selectable_value(&mut self.log_filter, LogFilter::Trace, "Trace");
                    });

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.auto_scroll_logs, "Auto-scroll");
                        if ui.button("Clear").clicked() {
                            self.log_entries.clear();
                        }
                    });
                    ui.separator();

                    let text_style = egui::TextStyle::Monospace;
                    let row_height = ui.text_style_height(&text_style);
                    let filtered: Vec<_> = self
                        .log_entries
                        .iter()
                        .filter(|e| self.filter_matches(e.level))
                        .collect();

                    egui::ScrollArea::vertical()
                        .auto_shrink([true, false])
                        .stick_to_bottom(self.auto_scroll_logs)
                        .show_rows(ui, row_height, filtered.len(), |ui, row_range| {
                            for i in row_range {
                                if let Some(entry) = filtered.get(i) {
                                    let color = Self::level_color(entry.level);
                                    let short_target = entry.target.split("::").last().unwrap_or(&entry.target);
                                    ui.horizontal(|ui| {
                                        ui.colored_label(color, format!("[{:5}]", entry.level));
                                        ui.colored_label(egui::Color32::GRAY, format!("{:>8}", short_target));
                                        ui.label(&entry.message);
                                    });
                                }
                            }
                        });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
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

                    if self.texture.is_none() {
                        self.core.load_rom(rom_path);
                    }

                    self.core.run_frame();

                    let rgba = self.core.framebuffer_rgba();
                    let size = [core::video::GBA_SCREEN_W, core::video::GBA_SCREEN_H];
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
                    ui.image((tex.id(), desired));
                }
            }
        });

        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let config = Config {
            recent_files: self.recent_files.clone(),
            bios_path: self.bios_path.clone(),
        };
        if let Err(e) = save_config(&config) {
            eprintln!("Failed to save config: {}", e);
        }
    }
}

fn main() -> eframe::Result<()> {
    let log_level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    let _ = core::log_buffer::init_logger(log_level);

    let args = Args::parse();
    let icon = IconData::default();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("RoBA - GBA Emulator")
            .with_app_id("com.roba.gba")
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "RoBA",
        native_options,
        Box::new(|_cc| Ok(Box::new(GbaApp::new(args.rom_path, args.bios)))),
    )
}
