mod buffer;
mod commands;
mod config;

use buffer::{TypewriterBuffer, ViewMode};
use commands::{parse_command, save_buffer, load_file, export_to_pdf, CommandResult};
use config::{Config, AppTheme};
use iced::widget::container::Appearance;

use iced::{
    event, executor,
    keyboard::{self, key::Named},
    widget::{column, container, text, Column, Space, row, button},
    window, Application, Color, Command, Element, Event, Length, Settings, Size, Subscription, Theme,
};

use rodio::{OutputStream, Sink, Source};

// Re-export or use smol_str directly as it's a dependency of iced
use iced::advanced::graphics::core::SmolStr;

fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.window.decorations = true;
    settings.window.transparent = false;
    settings.window.size = Size::new(1200.0, 800.0);
    settings.window.position = window::Position::Centered;
    settings.antialiasing = true;

    FocusWrite::run(settings)
}

// ─── Application State ────────────────────────────────────────────────────────

struct FocusWrite {
    buffer: TypewriterBuffer,
    config: Config,
    /// Caches the currently active font
    loaded_font: iced::Font,
    command_input: Option<String>,
    status_msg: Option<String>,
    caret_visible: bool,
    window_width: f32,
    _audio_stream: Option<(OutputStream, rodio::OutputStreamHandle)>,
    audio_sink: Option<Sink>,
    session_start: std::time::Instant,
    initial_word_count: usize,
    show_summary: bool,
}

#[derive(Debug, Clone)]
enum Message {
    KeyPressed(keyboard::Key, keyboard::Modifiers, Option<SmolStr>),
    CommandSubmitted,
    CommandCancelled,
    CaretTick,
    WindowResized(f32, f32),
    Undo,
    Redo,
    AutoSaveTick,
    QuitConfirmed,
    Copy,
    Cut,
    Paste,
    ClipboardPasted(Option<String>),
    SelectAll,
}

impl FocusWrite {
    fn resolve_font(name: &Option<String>) -> iced::Font {
        if let Some(ref name) = name {
            let leaked: &'static str = Box::leak(name.clone().into_boxed_str());
            iced::Font::with_name(leaked)
        } else {
            iced::Font::MONOSPACE
        }
    }

    fn play_click(&self) {
        if let Some(ref sink) = self.audio_sink {
            let source = rodio::source::SineWave::new(200.0)
                .take_duration(std::time::Duration::from_millis(35))
                .amplify(0.25);
            sink.append(source);
        }
    }

    fn play_ding(&self) {
        if let Some(ref sink) = self.audio_sink {
            let source = rodio::source::SineWave::new(450.0)
                .take_duration(std::time::Duration::from_millis(150))
                .amplify(0.2);
            sink.append(source);
        }
    }
}

// ─── Application Implementation ───────────────────────────────────────────────

impl Application for FocusWrite {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let config = Config::load();
        let buffer = TypewriterBuffer::new_with_config(config.decay_rate, config.min_opacity);
        let loaded_font = Self::resolve_font(&config.font_name);
        
        let mut status_msg = None;
        let (stream, sink) = match OutputStream::try_default() {
            Ok((s, handle)) => {
                match Sink::try_new(&handle) {
                    Ok(sink) => {
                        sink.set_volume(0.4);
                        (Some((s, handle)), Some(sink))
                    }
                    Err(e) => {
                        status_msg = Some(format!("Audio Sink Error: {}", e));
                        (Some((s, handle)), None)
                    }
                }
            }
            Err(e) => {
                status_msg = Some(format!("Audio Init Error: {}", e));
                (None, None)
            }
        };

        (
            FocusWrite {
                buffer,
                config,
                loaded_font,
                command_input: None,
                status_msg,
                caret_visible: true,
                window_width: 1200.0,
                _audio_stream: stream,
                audio_sink: sink,
                session_start: std::time::Instant::now(),
                initial_word_count: 0, // Will be updated if a file is loaded
                show_summary: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String { "Focus-Write".to_string() }

    fn subscription(&self) -> Subscription<Message> {
        let keyboard_sub = event::listen_with(|event, _status| {
            match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, text, .. }) => {
                    Some(Message::KeyPressed(key, modifiers, text))
                }
                Event::Window(_id, window::Event::Resized { width, height }) => {
                    Some(Message::WindowResized(width as f32, height as f32))
                }
                _ => None,
            }
        });

        let caret_sub = iced::time::every(std::time::Duration::from_millis(530))
            .map(|_| Message::CaretTick);

        let autosave_sub = iced::time::every(std::time::Duration::from_secs(60))
            .map(|_| Message::AutoSaveTick);

        Subscription::batch([keyboard_sub, caret_sub, autosave_sub])
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CaretTick => { self.caret_visible = !self.caret_visible; }
            Message::WindowResized(w, _h) => { self.window_width = w; }
            Message::KeyPressed(key, _mods, text) if self.command_input.is_some() => {
                match key {
                    keyboard::Key::Named(Named::Escape) => { return self.update(Message::CommandCancelled); }
                    keyboard::Key::Named(Named::Enter) => { return self.update(Message::CommandSubmitted); }
                    keyboard::Key::Named(Named::Backspace) => { if let Some(ref mut cmd) = self.command_input { cmd.pop(); } }
                    _ => { if let Some(s) = text { if let Some(ref mut cmd) = self.command_input { cmd.push_str(&s); } } }
                }
            }
            Message::KeyPressed(key, mods, text) => {
                match key {
                    keyboard::Key::Named(Named::Escape) => { self.status_msg = None; }
                    keyboard::Key::Named(Named::Backspace) => { 
                        if self.buffer.selection.is_some() { self.buffer.push_history(); }
                        self.buffer.delete_backwards(); 
                        self.caret_visible = true; 
                        self.play_click(); 
                        self.status_msg = None; 
                    }
                    keyboard::Key::Named(Named::Enter) => { 
                        self.buffer.push_history();
                        self.buffer.insert_newline(); 
                        self.caret_visible = true; 
                        self.play_ding(); 
                        self.status_msg = None; 
                    }
                    keyboard::Key::Named(Named::ArrowUp) => { 
                        let old_pos = (self.buffer.cursor_line, self.buffer.cursor_col);
                        if self.buffer.cursor_line > 0 { 
                            self.buffer.cursor_line -= 1; 
                            let line = &self.buffer.lines[self.buffer.cursor_line];
                            if !line.is_char_boundary(self.buffer.cursor_col) {
                                let mut new_col = self.buffer.cursor_col;
                                while new_col > 0 && !line.is_char_boundary(new_col) {
                                    new_col -= 1;
                                }
                                self.buffer.cursor_col = new_col;
                            }
                            self.buffer.cursor_col = self.buffer.cursor_col.min(line.len()); 
                        } 
                        self.handle_selection_move(mods.shift(), old_pos);
                        self.status_msg = None;
                    }
                    keyboard::Key::Named(Named::ArrowDown) => { 
                        let old_pos = (self.buffer.cursor_line, self.buffer.cursor_col);
                        if self.buffer.cursor_line + 1 < self.buffer.lines.len() { 
                            self.buffer.cursor_line += 1; 
                            let line = &self.buffer.lines[self.buffer.cursor_line];
                            if !line.is_char_boundary(self.buffer.cursor_col) {
                                let mut new_col = self.buffer.cursor_col;
                                while new_col > 0 && !line.is_char_boundary(new_col) {
                                    new_col -= 1;
                                }
                                self.buffer.cursor_col = new_col;
                            }
                            self.buffer.cursor_col = self.buffer.cursor_col.min(line.len()); 
                        } 
                        self.handle_selection_move(mods.shift(), old_pos);
                        self.status_msg = None;
                    }
                    keyboard::Key::Named(Named::ArrowLeft) => { 
                        let old_pos = (self.buffer.cursor_line, self.buffer.cursor_col);
                        if self.buffer.cursor_col > 0 { 
                            let line = &self.buffer.lines[self.buffer.cursor_line]; 
                            if let Some((idx, _)) = line[..self.buffer.cursor_col].char_indices().last() { self.buffer.cursor_col = idx; } 
                        } else if self.buffer.cursor_line > 0 { 
                            self.buffer.cursor_line -= 1; 
                            self.buffer.cursor_col = self.buffer.lines[self.buffer.cursor_line].len(); 
                        } 
                        self.handle_selection_move(mods.shift(), old_pos);
                        self.status_msg = None; 
                    }
                    keyboard::Key::Named(Named::ArrowRight) => { 
                        let old_pos = (self.buffer.cursor_line, self.buffer.cursor_col);
                        let line = &self.buffer.lines[self.buffer.cursor_line]; 
                        if self.buffer.cursor_col < line.len() { 
                            let ch = line[self.buffer.cursor_col..].chars().next().unwrap(); 
                            self.buffer.cursor_col += ch.len_utf8(); 
                        } else if self.buffer.cursor_line + 1 < self.buffer.lines.len() { 
                            self.buffer.cursor_line += 1; 
                            self.buffer.cursor_col = 0; 
                        } 
                        self.handle_selection_move(mods.shift(), old_pos);
                        self.status_msg = None; 
                    }
                    keyboard::Key::Named(Named::Home) => { 
                        let old_pos = (self.buffer.cursor_line, self.buffer.cursor_col);
                        self.buffer.cursor_col = 0; 
                        self.handle_selection_move(mods.shift(), old_pos);
                        self.status_msg = None; 
                    }
                    keyboard::Key::Named(Named::End) => { 
                        let old_pos = (self.buffer.cursor_line, self.buffer.cursor_col);
                        self.buffer.cursor_col = self.buffer.lines[self.buffer.cursor_line].len(); 
                        self.handle_selection_move(mods.shift(), old_pos);
                        self.status_msg = None; 
                    }
                    keyboard::Key::Named(Named::Space) => { 
                        if !mods.control() && !mods.alt() { 
                            self.buffer.push_history();
                            self.buffer.insert_char(' '); 
                            self.caret_visible = true; 
                            self.play_click(); 
                            self.status_msg = None; 
                        } 
                    }
                    keyboard::Key::Named(Named::Tab) => { 
                        self.buffer.push_history();
                        for _ in 0..4 { self.buffer.insert_char(' '); } 
                        self.play_click(); 
                        self.status_msg = None; 
                    }
                    keyboard::Key::Character(ref s) => {
                        let key_char = s.to_lowercase();
                        if mods.control() && key_char == ";" { self.command_input = Some("".to_string()); }
                        else if mods.control() && key_char == "s" { if mods.shift() || self.buffer.file_path.is_none() { self.command_input = Some("w ".to_string()); } else { self.do_save(None); } }
                        else if mods.control() && key_char == "o" { self.command_input = Some("e ".to_string()); }
                        else if mods.control() && key_char == "v" { return self.update(Message::Paste); }
                        else if mods.control() && key_char == "c" { return self.update(Message::Copy); }
                        else if mods.control() && key_char == "x" { return self.update(Message::Cut); }
                        else if mods.control() && key_char == "a" { return self.update(Message::SelectAll); }
                        else if mods.control() && key_char == "z" { return self.update(Message::Undo); }
                        else if mods.control() && key_char == "y" { return self.update(Message::Redo); }
                        else if mods.control() && key_char == "t" { self.config.theme = self.config.theme.next(); self.config.save(); }
                        else if mods.control() && key_char == "f" { self.config.next_font(); self.loaded_font = Self::resolve_font(&self.config.font_name); self.config.save(); }
                        else if mods.control() && (key_char == "=" || key_char == "+") { self.config.font_size = (self.config.font_size + 2.0).min(72.0); self.config.save(); }
                        else if mods.control() && key_char == "-" { self.config.font_size = (self.config.font_size - 2.0).max(8.0); self.config.save(); }
                        else if let Some(t) = text { if !mods.control() && !mods.alt() && self.command_input.is_none() { 
                            if t.contains(' ') || self.buffer.selection.is_some() { self.buffer.push_history(); }
                            for ch in t.chars() { self.buffer.insert_char(ch); } 
                            self.caret_visible = true; 
                            self.play_click(); 
                            self.status_msg = None; 
                        } }
                    }
                    _ => { if let Some(t) = text { if !mods.control() && !mods.alt() && self.command_input.is_none() { 
                        if t.contains(' ') || self.buffer.selection.is_some() { self.buffer.push_history(); }
                        for ch in t.chars() { self.buffer.insert_char(ch); } 
                        self.caret_visible = true; 
                        self.play_click(); 
                        self.status_msg = None; 
                    } } }
                }
            }
            Message::CommandCancelled => { 
                self.command_input = None; 
                self.show_summary = false;
            }
            Message::CommandSubmitted => {
                let input = self.command_input.take().unwrap_or_default();
                let result = parse_command(&input, &self.buffer, &self.config);
                match result {
                    CommandResult::Quit => { self.show_summary = true; }
                    CommandResult::ToggleView => { self.buffer.toggle_view(); }
                    CommandResult::Save(path) => { self.do_save(Some(path)); }
                    CommandResult::ExportPdf(path) => {
                        match export_to_pdf(&self.buffer, &path) {
                            Ok(_) => { self.status_msg = Some(format!("PDF Exported: {}", path.display())); }
                            Err(e) => { self.status_msg = Some(e); }
                        }
                    }
                    CommandResult::Open(path) => { 
                        match load_file(&path) {
                            Ok(mut buf) => {
                                buf.update_config(self.config.decay_rate, self.config.min_opacity);
                                self.buffer = buf;
                                self.buffer.file_path = Some(path);
                                self.initial_word_count = self.buffer.lines.iter().flat_map(|l| l.split_whitespace()).count();
                                self.status_msg = Some("File loaded".to_string());
                            }
                            Err(e) => {
                                self.status_msg = Some(format!("Load Error: {}", e));
                            }
                        }
                    }
                    CommandResult::SetTheme(theme_name) => {
                        match theme_name.to_lowercase().as_str() {
                            "default" => self.config.theme = AppTheme::Default,
                            "sepia" => self.config.theme = AppTheme::Sepia,
                            "eink" | "ink" => self.config.theme = AppTheme::EInk,
                            "night" | "dark" => self.config.theme = AppTheme::Night,
                            "amoled" | "black" => self.config.theme = AppTheme::Amoled,
                            _ => { self.status_msg = Some(format!("Unknown theme: {}. Available: default, sepia, eink, night, amoled", theme_name)); }
                        }
                        self.config.save();
                    }
                    CommandResult::SetFont(font_name) => {
                        self.config.font_name = if font_name == "default" { None } else { Some(font_name) };
                        self.loaded_font = Self::resolve_font(&self.config.font_name);
                        self.config.save();
                    }
                    CommandResult::SetFontSize(s) => { self.config.font_size = s; self.config.save(); }
                    CommandResult::SetLineHeight(lh) => { self.config.line_height = lh; self.config.save(); }
                    CommandResult::SetMaxWidth(mw) => { self.config.max_width = mw; self.config.save(); }
                    CommandResult::SetDecay(dr) => { self.config.decay_rate = dr; self.buffer.update_config(dr, self.config.min_opacity); self.config.save(); }
                    CommandResult::SetOpacity(mo) => { self.config.min_opacity = mo; self.buffer.update_config(self.config.decay_rate, mo); self.config.save(); }
                    CommandResult::SetSavePath(p) => { self.config.default_save_path = p; self.config.save(); }
                    CommandResult::ResetSettings => {
                        self.config = Config::default();
                        self.loaded_font = Self::resolve_font(&self.config.font_name);
                        self.buffer.update_config(self.config.decay_rate, self.config.min_opacity);
                        self.config.save();
                        self.status_msg = Some("Settings reset to default".to_string());
                    }
                    CommandResult::Error(e) => { self.status_msg = Some(e); }
                }
            }
            Message::Undo => { self.buffer.undo(); self.status_msg = Some("Undo".to_string()); }
            Message::Redo => { self.buffer.redo(); self.status_msg = Some("Redo".to_string()); }
            Message::AutoSaveTick => {
                let backup_path = if let Some(ref path) = self.buffer.file_path {
                    let mut p = path.clone();
                    p.set_extension("backup");
                    p
                } else {
                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    std::path::PathBuf::from(home).join("focus.backup")
                };
                let _ = save_buffer(&self.buffer, &backup_path);
            }
            Message::QuitConfirmed => { return window::close(window::Id::MAIN); }
            Message::Copy => {
                if let Some(text) = self.buffer.get_selected_text() {
                    return iced::clipboard::write(text);
                }
            }
            Message::Cut => {
                if let Some(text) = self.buffer.get_selected_text() {
                    self.buffer.push_history();
                    self.buffer.delete_selection();
                    return iced::clipboard::write(text);
                }
            }
            Message::Paste => {
                return iced::clipboard::read(Message::ClipboardPasted);
            }
            Message::ClipboardPasted(content) => {
                if let Some(text) = content {
                    self.buffer.push_history();
                    for ch in text.chars() {
                        self.buffer.insert_char(ch);
                    }
                }
            }
            Message::SelectAll => {
                self.buffer.select_all();
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let bg = self.config.bg_color();
        let text_color = self.config.text_rgb();
        let caret_color = self.config.caret_rgb();

        if self.show_summary {
            return container(self.view_summary(self.loaded_font, caret_color))
                .width(Length::Fill).height(Length::Fill)
                .style(iced::theme::Container::Custom(Box::new(BgStyle(bg)))).into();
        }

        // Add padding that scales or is fixed, ensuring it doesn't collapse small windows
        let editor_area = container(self.build_text_view(text_color))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20) // Moderate padding for all modes
            .center_x()
            .center_y();

        let status_text = if let Some(ref msg) = self.status_msg {
            text(msg).size(13).font(self.loaded_font).style(iced::theme::Text::Color(Color { a: 0.7, ..text_color }))
        } else {
            let mode = match self.buffer.mode { ViewMode::Focused => "FOCUS", ViewMode::Full => "FULL" };
            let word_count = self.buffer.lines.iter().flat_map(|l| l.split_whitespace()).count();
            let reading_time = (word_count as f32 / 225.0).ceil() as u32;
            text(format!("{} · {} words · {} min read", mode, word_count, reading_time))
                .size(12).font(self.loaded_font).style(iced::theme::Text::Color(Color { a: 0.3, ..text_color }))
        };

        let command_bar: Element<'_, Message> = if let Some(ref cmd) = self.command_input {
            let (label, display) = if cmd.starts_with("w ") { ("SAVE AS: ", &cmd[2..]) }
                else if cmd.starts_with("e ") || cmd.starts_with("o ") { ("OPEN: ", &cmd[2..]) }
                else { (":", cmd.as_str()) };
            container(text(format!("{}{}", label, display)).size(18).font(self.loaded_font).style(iced::theme::Text::Color(caret_color))).padding(20).into()
        } else {
            Space::with_height(0).into()
        };

        let content = Column::new()
            .push(editor_area)
            .push(command_bar)
            .push(container(status_text).padding(20).width(Length::Fill))
            .width(Length::Fill).height(Length::Fill);

        container(content)
            .width(Length::Fill).height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(BgStyle(bg)))).into()
    }
}

impl FocusWrite {
    fn build_text_view(&self, base_text_color: Color) -> Element<'_, Message> {
        let buf = &self.buffer;
        let window_size = 15; 
        let half_window = window_size / 2;
        let font_size = self.config.font_size;
        let is_focused = matches!(buf.mode, ViewMode::Focused);
        
        // Column spacing is (L - 1) * F
        let spacing = (font_size * (self.config.line_height - 1.0)) as u16;

        let mut col = Column::new()
            .spacing(spacing)
            .width(Length::Fill)
            .align_items(iced::Alignment::Start);

        let selection_range = buf.get_selection_range();

        for i in 0..window_size {
            let logical_idx = buf.cursor_line as isize - half_window as isize + i as isize;
            
            if logical_idx < 0 || logical_idx >= buf.lines.len() as isize {
                col = col.push(Space::with_height(font_size as u16));
                continue;
            }

            let idx = logical_idx as usize;
            let line_str = &buf.lines[idx];
            let dist = (idx as isize - buf.cursor_line as isize).abs() as usize;
            let opacity = if is_focused { buf.line_opacity(dist) } else { 1.0 };
            let color = Color { a: opacity, ..base_text_color };

            // Determine selection on this line
            let line_selection = selection_range.and_then(|((s_l, s_c), (e_l, e_c))| {
                if idx < s_l || idx > e_l { None }
                else if s_l == e_l { Some((s_c, e_c, e_l)) }
                else if idx == s_l { Some((s_c, line_str.len(), e_l)) }
                else if idx == e_l { Some((0, e_c, e_l)) }
                else { Some((0, line_str.len(), e_l)) }
            });

            if let Some((sel_start, sel_end, sel_e_l)) = line_selection {
                let mut line_row = row![].spacing(0);
                
                // Part before selection
                if sel_start > 0 {
                    line_row = line_row.push(text(&line_str[..sel_start]).size(font_size).font(self.loaded_font).style(iced::theme::Text::Color(color)));
                }

                // Selected part
                let sel_text = &line_str[sel_start..sel_end];
                let disp_sel = if sel_text.is_empty() && idx != sel_e_l { " " } else { sel_text };
                line_row = line_row.push(
                    container(text(disp_sel).size(font_size).font(self.loaded_font).style(iced::theme::Text::Color(self.config.bg_color())))
                        .style(iced::theme::Container::Custom(Box::new(SelectionStyle(color))))
                );

                // Part after selection
                if sel_end < line_str.len() {
                    line_row = line_row.push(text(&line_str[sel_end..]).size(font_size).font(self.loaded_font).style(iced::theme::Text::Color(color)));
                }

                col = col.push(line_row);
            } else {
                let content = if idx == buf.cursor_line {
                    let caret = if self.caret_visible { "│" } else { " " };
                    let pos = buf.cursor_col.min(line_str.len());
                    format!("{}{}{}", &line_str[..pos], caret, &line_str[pos..])
                } else {
                    if line_str.is_empty() { " ".to_string() } else { line_str.clone() }
                };

                col = col.push(
                    text(content)
                        .size(font_size)
                        .font(self.loaded_font)
                        .width(Length::Fill)
                        .horizontal_alignment(iced::alignment::Horizontal::Left)
                        .style(iced::theme::Text::Color(color))
                );
            }
        }

        container(col)
            .width(Length::Fill)
            .max_width(self.config.max_width)
            .into()
    }

    fn view_summary(&self, font: iced::Font, accent: Color) -> Element<'_, Message> {
        let current_word_count = self.buffer.lines.iter().flat_map(|l| l.split_whitespace()).count();
        let session_words = current_word_count.saturating_sub(self.initial_word_count);
        let duration = self.session_start.elapsed();
        let minutes = duration.as_secs_f32() / 60.0;
        let wpm = if minutes > 0.1 { (session_words as f32 / minutes) as u32 } else { 0 };

        let title = text("SESSION COMPLETE").size(40).font(font).style(iced::theme::Text::Color(accent));
        
        let stats = Column::new().spacing(20).push(
            text(format!("Words written: {}", session_words)).size(24).font(font)
        ).push(
            text(format!("Time elapsed: {}m {}s", duration.as_secs() / 60, duration.as_secs() % 60)).size(24).font(font)
        ).push(
            text(format!("Average speed: {} WPM", wpm)).size(24).font(font)
        );

        let quit_btn = button(container(text("Quit Application").font(font)).padding(12)).on_press(Message::QuitConfirmed).style(iced::theme::Button::Destructive);
        let continue_btn = button(container(text("Keep Writing").font(font)).padding(12)).on_press(Message::CommandCancelled); // Reuse cancelled to hide overlay

        container(
            column![
                title,
                Space::with_height(40),
                stats,
                Space::with_height(60),
                row![continue_btn, Space::with_width(20), quit_btn],
            ].max_width(600).align_items(iced::Alignment::Center)
        )
        .padding(60)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn do_save(&mut self, path: Option<std::path::PathBuf>) {
        let path = path.or_else(|| self.buffer.file_path.clone()).unwrap_or_else(|| std::path::PathBuf::from(&self.config.default_save_path));
        match save_buffer(&self.buffer, &path) {
            Ok(_) => {
                self.buffer.file_path = Some(path.clone());
                self.status_msg = Some(format!("Saved: {}", path.display()));
            }
            Err(e) => {
                self.status_msg = Some(format!("Save Error: {}", e));
            }
        }
    }

    fn handle_selection_move(&mut self, shift_held: bool, old_pos: (usize, usize)) {
        if shift_held {
            if let Some((start, _)) = self.buffer.selection {
                self.buffer.selection = Some((start, (self.buffer.cursor_line, self.buffer.cursor_col)));
            } else {
                self.buffer.selection = Some((old_pos, (self.buffer.cursor_line, self.buffer.cursor_col)));
            }
        } else {
            self.buffer.selection = None;
        }
    }
}

struct BgStyle(Color);
impl iced::widget::container::StyleSheet for BgStyle {
    type Style = Theme;
    fn appearance(&self, _: &Self::Style) -> Appearance {
        Appearance { background: Some(iced::Background::Color(self.0)), ..Default::default() }
    }
}

struct SelectionStyle(Color);
impl iced::widget::container::StyleSheet for SelectionStyle {
    type Style = Theme;
    fn appearance(&self, _: &Self::Style) -> Appearance {
        Appearance { background: Some(iced::Background::Color(self.0)), ..Default::default() }
    }
}
