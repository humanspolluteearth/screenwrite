use std::path::PathBuf;

use crate::buffer::TypewriterBuffer;
use crate::config::Config;

#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    Save(PathBuf),
    Open(PathBuf),
    ExportPdf(PathBuf),
    ToggleView,
    Quit,
    SetTheme(String),
    SetFont(String),
    SetFontSize(f32),
    SetLineHeight(f32),
    SetMaxWidth(f32),
    SetDecay(f32),
    SetOpacity(f32),
    SetSavePath(String),
    ResetSettings,
    Error(String),
}

/// Parses and executes the current command string in the overlay.
/// Returns a CommandResult describing what should happen.
pub fn parse_command(input: &str, buffer: &TypewriterBuffer, config: &Config) -> CommandResult {
    let input = input.trim();

    if input == "q" || input == "quit" {
        return CommandResult::Quit;
    }

    if input == "all" {
        return CommandResult::ToggleView;
    }

    if input == "reset" {
        return CommandResult::ResetSettings;
    }

    if input == "pdf" || input == "export" {
        let mut path = current_file_path(buffer, config);
        path.set_extension("pdf");
        return CommandResult::ExportPdf(path);
    }

    // :w  -> saves to current file if set, else to default_save_path
    if input == "w" {
        let path = current_file_path(buffer, config);
        return CommandResult::Save(path);
    }

    // :w <path>
    if let Some(rest) = input.strip_prefix("w ") {
        let path = ensure_txt_extension(expand_home(rest.trim()));
        return CommandResult::Save(path);
    }

    // :e <path> / :o <path>
    if let Some(rest) = input.strip_prefix("e ").or_else(|| input.strip_prefix("o ")) {
        let path = ensure_txt_extension(expand_home(rest.trim()));
        return CommandResult::Open(path);
    }

    // Configuration commands
    if let Some(rest) = input.strip_prefix("theme ") {
        return CommandResult::SetTheme(rest.trim().to_string());
    }
    if let Some(rest) = input.strip_prefix("font ") {
        return CommandResult::SetFont(rest.trim().to_string());
    }
    if let Some(rest) = input.strip_prefix("size ") {
        if let Ok(val) = rest.trim().parse::<f32>() { return CommandResult::SetFontSize(val); }
    }
    if let Some(rest) = input.strip_prefix("lh ") {
        if let Ok(val) = rest.trim().parse::<f32>() { return CommandResult::SetLineHeight(val); }
    }
    if let Some(rest) = input.strip_prefix("width ") {
        if let Ok(val) = rest.trim().parse::<f32>() { return CommandResult::SetMaxWidth(val); }
    }
    if let Some(rest) = input.strip_prefix("decay ") {
        if let Ok(val) = rest.trim().parse::<f32>() { return CommandResult::SetDecay(val); }
    }
    if let Some(rest) = input.strip_prefix("opacity ") {
        if let Ok(val) = rest.trim().parse::<f32>() { return CommandResult::SetOpacity(val); }
    }
    if let Some(rest) = input.strip_prefix("savepath ") {
        return CommandResult::SetSavePath(rest.trim().to_string());
    }

    if input == "about" || input == "help" {
        return CommandResult::Error("Focus-Write v0.1.0 - A Retro Minimalist Editor. Commands: :w, :q, :e, :theme, :font, :size, :lh, :width, :decay, :opacity, :savepath, :reset".to_string());
    }

    CommandResult::Error(format!("Unknown command: {}", input))
}

pub fn save_buffer(buffer: &TypewriterBuffer, path: &PathBuf) -> Result<(), String> {
    let text = buffer.lines.join("\n");
    std::fs::write(path, text).map_err(|e| e.to_string())
}

pub fn load_file(path: &PathBuf) -> Result<TypewriterBuffer, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let lines: Vec<String> = if content.is_empty() {
        vec![String::new()]
    } else {
        content.lines().map(|l| l.to_string()).collect()
    };

    let mut buf = TypewriterBuffer::new();
    buf.lines = lines;
    buf.cursor_line = buf.lines.len().saturating_sub(1);
    buf.cursor_col = buf.lines.last().map(|l| l.len()).unwrap_or(0);
    Ok(buf)
}

pub fn export_to_pdf(buffer: &TypewriterBuffer, path: &PathBuf) -> Result<(), String> {
    let font_locations = [
        ("/usr/share/fonts/liberation", "LiberationMono"),
        ("/usr/share/fonts/TTF", "DejaVuSansMono"),
        ("/usr/share/fonts/truetype/liberation", "LiberationMono"),
    ];

    let mut font_family = None;

    for (dir, name) in font_locations {
        if std::path::Path::new(dir).exists() {
            if let Ok(family) = genpdf::fonts::from_files(dir, name, None) {
                font_family = Some(family);
                break;
            }
        }
    }

    let font_family = font_family.ok_or_else(|| {
        "No suitable monospace system font found. Please install ttf-liberation or ttf-dejavu.".to_string()
    })?;

    let mut doc = genpdf::Document::new(font_family);
    doc.set_title("Focus-Write Export");

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    for line in &buffer.lines {
        let l = if line.is_empty() { " " } else { line };
        doc.push(genpdf::elements::Text::new(l));
    }

    doc.render_to_file(path).map_err(|e| format!("PDF Render Error: {}", e))?;
    Ok(())
}

fn current_file_path(buffer: &TypewriterBuffer, config: &Config) -> PathBuf {
    buffer.file_path.clone().unwrap_or_else(|| {
        expand_home(&config.default_save_path)
    })
}

fn expand_home(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(&path[2..])
    } else {
        PathBuf::from(path)
    }
}

fn ensure_txt_extension(mut path: PathBuf) -> PathBuf {
    if path.extension().is_none() {
        path.set_extension("txt");
    }
    path
}
