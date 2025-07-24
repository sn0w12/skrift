use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use fltk::{
    enums::{Color, Key, Shortcut},
    app
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Binding {
    Save,
    Quit,
    Reload,
    MoveLineUp,
    MoveLineDown,
}

impl Binding {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "save" => Some(Binding::Save),
            "quit" => Some(Binding::Quit),
            "reload" => Some(Binding::Reload),
            "move_line_up" => Some(Binding::MoveLineUp),
            "move_line_down" => Some(Binding::MoveLineDown),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cursor {
    Normal = 0,
    Caret = 1,
    Dim = 2,
    Block = 3,
    Heavy = 4,
    Simple = 5,
}

impl Cursor {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "normal" => Cursor::Normal,
            "caret" => Cursor::Caret,
            "dim" => Cursor::Dim,
            "block" => Cursor::Block,
            "heavy" => Cursor::Heavy,
            "simple" => Cursor::Simple,
            _ => Cursor::Simple,
        }
    }

    pub fn to_fltk_cursor(self) -> fltk::text::Cursor {
        match self {
            Cursor::Normal => fltk::text::Cursor::Normal,
            Cursor::Caret => fltk::text::Cursor::Caret,
            Cursor::Dim => fltk::text::Cursor::Dim,
            Cursor::Block => fltk::text::Cursor::Block,
            Cursor::Heavy => fltk::text::Cursor::Heavy,
            Cursor::Simple => fltk::text::Cursor::Simple,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub bindings: HashMap<Binding, String>,
    pub theme: Theme,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub background: String,
    pub foreground: String,
    pub font_family: String,
    pub font_size: i32,
    pub selection_color: String,
    pub cursor_flash: bool,
    pub cursor_flash_interval: f64,
    pub negative_color: String,
    pub cursor_style: Cursor,
}

impl Config {
    pub fn load() -> Self {
        let mut path = dirs::home_dir().unwrap_or(PathBuf::from("."));
        path.push(".config/skrift/config.skrift");

        if path.exists() {
            if let Ok(text) = fs::read_to_string(&path) {
                if let Ok(value) = toml::from_str::<toml::Value>(&text) {
                    return Config::from_toml(value);
                }
            }
        }

        Self::default()
    }

    fn from_toml(value: toml::Value) -> Self {
        let default = Config::default();

        let bindings = value.get("bindings")
            .and_then(|v| v.as_table())
            .map(|tbl| {
                tbl.iter()
                    .filter_map(|(k, v)| {
                        Binding::from_str(k).map(|b| (b, v.as_str().unwrap_or("").to_string()))
                    })
                    .collect()
            })
            .unwrap_or_else(|| default.bindings.clone());

        let theme = value.get("theme").and_then(|t| t.as_table());

        fn get_str(theme: Option<&toml::value::Table>, key: &str, default: &str) -> String {
            theme.and_then(|t| t.get(key)).and_then(|v| v.as_str()).unwrap_or(default).to_string()
        }
        fn get_i32(theme: Option<&toml::value::Table>, key: &str, default: i32) -> i32 {
            theme.and_then(|t| t.get(key)).and_then(|v| v.as_integer()).map(|i| i as i32).unwrap_or(default)
        }
        fn get_bool(theme: Option<&toml::value::Table>, key: &str, default: bool) -> bool {
            theme.and_then(|t| t.get(key)).and_then(|v| v.as_bool()).unwrap_or(default)
        }
        fn get_f64(theme: Option<&toml::value::Table>, key: &str, default: f64) -> f64 {
            theme.and_then(|t| t.get(key)).and_then(|v| v.as_float()).unwrap_or(default)
        }
        fn get_cursor(theme: Option<&toml::value::Table>, key: &str, default: Cursor) -> Cursor {
            theme.and_then(|t| t.get(key)).and_then(|v| v.as_str()).map(Cursor::from_str).unwrap_or(default)
        }

        Config {
            bindings,
            theme: Theme {
                background: get_str(theme, "background", &default.theme.background),
                foreground: get_str(theme, "foreground", &default.theme.foreground),
                font_family: get_str(theme, "font_family", &default.theme.font_family),
                font_size: get_i32(theme, "font_size", default.theme.font_size),
                selection_color: get_str(theme, "selection_color", &default.theme.selection_color),
                cursor_flash: get_bool(theme, "cursor_flash", default.theme.cursor_flash),
                cursor_flash_interval: get_f64(theme, "cursor_flash_interval", default.theme.cursor_flash_interval),
                negative_color: get_str(theme, "negative_color", &default.theme.negative_color),
                cursor_style: get_cursor(theme, "cursor_style", default.theme.cursor_style),
            },
        }
    }

    pub fn default() -> Self {
        Self {
            bindings: HashMap::from([
                (Binding::Save, "Ctrl+S".into()),
                (Binding::Quit, "Ctrl+Q".into()),
                (Binding::Reload, "Ctrl+R".into()),
                (Binding::MoveLineUp, "Alt+Up".into()),
                (Binding::MoveLineDown, "Alt+Down".into()),
            ]),
            theme: Theme {
                background: "#1e1e1e".into(),
                foreground: "#c0c0c0".into(),
                font_family: "Courier".into(),
                font_size: 16,
                selection_color: "#51a9e8ff".into(),
                cursor_flash: true,
                cursor_flash_interval: 0.5,
                negative_color: "#FF0000".into(),
                cursor_style: Cursor::Simple,
            }
        }
    }

    pub fn shortcut_matches(shortcut: &str) -> bool {
        let parts: Vec<&str> = shortcut.split('+').collect();
        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;
        let mut key: Option<Key> = None;
        let mut char_key: Option<char> = None;

        for part in parts {
            match part {
                "Ctrl" => ctrl = true,
                "Alt" => alt = true,
                "Shift" => shift = true,
                "Left" => key = Some(Key::Left),
                "Right" => key = Some(Key::Right),
                "Up" => key = Some(Key::Up),
                "Down" => key = Some(Key::Down),
                k if k.len() == 1 => {
                    char_key = Some(k.chars().next().unwrap().to_ascii_lowercase());
                }
                _ => {}
            }
        }

        let event_key = app::event_key();
        let event_state = app::event_state();

        if matches!(
            event_key,
            Key::ControlL | Key::ControlR | Key::AltL | Key::AltR | Key::ShiftL | Key::ShiftR
        ) {
            return false;
        }

        if let Some(k) = key {
            return event_key == k
                && ctrl == event_state.contains(Shortcut::Ctrl)
                && alt == event_state.contains(Shortcut::Alt)
                && shift == event_state.contains(Shortcut::Shift);
        }

        if let Some(c) = char_key {
            if matches!(
                event_key,
                Key::Left | Key::Right | Key::Up | Key::Down
                    | Key::F1 | Key::F2 | Key::F3 | Key::F4 | Key::F5 | Key::F6
                    | Key::F7 | Key::F8 | Key::F9 | Key::F10 | Key::F11 | Key::F12
            ) {
                return false;
            }
            let event_char = event_key.to_char().map(|ch| ch.to_ascii_lowercase());
            return event_char == Some(c)
                && ctrl == event_state.contains(Shortcut::Ctrl)
                && alt == event_state.contains(Shortcut::Alt)
                && shift == event_state.contains(Shortcut::Shift);
        }

        false
    }
}

impl Theme {
    pub fn color_from_str(&self, color: &str) -> Color {
        let hex = color.trim_start_matches("0x").trim_start_matches('#');
        Color::from_hex(u32::from_str_radix(hex, 16).unwrap_or(0xFFFFFF))
    }
}
