use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppTheme {
    Default,
    Sepia,
    EInk,
    Night,
    Amoled,
}

impl AppTheme {
    pub const ALL: [AppTheme; 5] = [
        AppTheme::Default,
        AppTheme::Sepia,
        AppTheme::EInk,
        AppTheme::Night,
        AppTheme::Amoled,
    ];

    pub fn colors(&self) -> (String, String, String) {
        match self {
            AppTheme::Default => ("#0d0d0d".to_string(), "#e0e0e0".to_string(), "#e2b714".to_string()),
            AppTheme::Sepia => ("#f4ecd8".to_string(), "#433422".to_string(), "#a6603a".to_string()),
            AppTheme::EInk => ("#ffffff".to_string(), "#000000".to_string(), "#000000".to_string()),
            AppTheme::Night => ("#0a0e14".to_string(), "#b3b1ad".to_string(), "#ffb454".to_string()),
            AppTheme::Amoled => ("#000000".to_string(), "#ffffff".to_string(), "#ffffff".to_string()),
        }
    }
    
    pub fn next(&self) -> Self {
        match self {
            AppTheme::Default => AppTheme::Sepia,
            AppTheme::Sepia => AppTheme::EInk,
            AppTheme::EInk => AppTheme::Night,
            AppTheme::Night => AppTheme::Amoled,
            AppTheme::Amoled => AppTheme::Default,
        }
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: AppTheme,
    pub font_size: f32,
    pub line_height: f32,
    pub max_width: f32,
    pub decay_rate: f32,
    pub min_opacity: f32,
    pub font_name: Option<String>,
    pub default_save_path: String,
}

pub const AVAILABLE_FONTS: &[&str] = &[
    "Monospace",
    "JetBrains Mono",
    "Fira Code",
    "IBM Plex Mono",
    "Courier Prime",
    "Liberation Mono",
    "Source Code Pro",
];

impl Default for Config {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let default_path = PathBuf::from(home).join("focus.txt").to_string_lossy().to_string();

        Self {
            theme: AppTheme::Default,
            font_size: 22.0,
            line_height: 1.8,
            max_width: 800.0,
            decay_rate: 0.35,
            min_opacity: 0.0,
            font_name: None,
            default_save_path: default_path,
        }
    }
}

impl Config {
    pub fn next_font(&mut self) {
        let current = self.font_name.as_deref().unwrap_or("Monospace");
        let idx = AVAILABLE_FONTS.iter().position(|&f| f == current).unwrap_or(0);
        let next_idx = (idx + 1) % AVAILABLE_FONTS.len();
        
        let next_font = AVAILABLE_FONTS[next_idx];
        self.font_name = if next_font == "Monospace" {
            None
        } else {
            Some(next_font.to_string())
        };
    }

    pub fn load() -> Self {
        let path = config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    serde_yaml::from_str(&contents).unwrap_or_default()
                }
                Err(_) => Self::default(),
            }
        } else {
            let cfg = Self::default();
            cfg.save();
            cfg
        }
    }

    pub fn save(&self) {
        let path = config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(yaml) = serde_yaml::to_string(self) {
            let _ = std::fs::write(path, yaml);
        }
    }

    pub fn bg_color(&self) -> iced::Color {
        let (bg, _, _) = self.theme.colors();
        parse_hex_color(&bg).unwrap_or(iced::Color::BLACK)
    }

    pub fn text_rgb(&self) -> iced::Color {
        let (_, text, _) = self.theme.colors();
        parse_hex_color(&text).unwrap_or(iced::Color::WHITE)
    }

    pub fn caret_rgb(&self) -> iced::Color {
        let (_, _, caret) = self.theme.colors();
        parse_hex_color(&caret).unwrap_or(iced::Color::WHITE)
    }
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("focus-write")
        .join("config.yaml")
}

pub fn parse_hex_color(hex: &str) -> Option<iced::Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        Some(iced::Color::from_rgb(r, g, b))
    } else {
        None
    }
}
