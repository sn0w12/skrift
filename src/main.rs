mod config;
mod config_watcher;
mod help;
mod status_dot;
mod widgets {
    pub mod dot;
    pub mod scrollbar;
}
use config::{Config, Binding};
use status_dot::{StatusDotState, update_status_dot, show_status_dot_timed, refresh_status_dot};

use fltk::{
    app, window::Window, text::TextEditor, text::TextBuffer,
    enums::{Font},
    prelude::*,
    frame::Frame,
};
use fltk_theme::{WidgetScheme, SchemeType};
use std::cell::RefCell;
use std::rc::Rc;
use std::env;
use std::fs;
use std::sync::mpsc::channel;

#[cfg(target_os = "linux")]
mod fontconfig_init {
    #[link(name = "fontconfig")]
    unsafe extern "C" {
        pub fn FcInit() -> ::std::os::raw::c_int;
    }
    pub fn init() {
        unsafe { FcInit(); }
    }
}
#[cfg(not(target_os = "linux"))]
mod fontconfig_init {
    pub fn init() {}
}

fn load_config_and_apply(
    cfg: &Rc<RefCell<Config>>,
    editor: &Rc<RefCell<TextEditor>>,
    wind: &mut Window,
    header: &mut Frame,
    blink_state: Rc<RefCell<bool>>,
    blink_paused: Rc<RefCell<bool>>,
    blink_timeout_handle: Rc<RefCell<Option<app::TimeoutHandle>>>,
    blink_callback: Rc<RefCell<Option<Box<dyn FnMut(app::TimeoutHandle)>>>>,
    editor_clone: Rc<RefCell<TextEditor>>,
    status_label: &mut Frame,
    dot: Option<&mut widgets::dot::Dot>,
    scrollbar: Option<&mut widgets::scrollbar::ScrollBar>,
) {
    let new_cfg = Config::load();
    *cfg.borrow_mut() = new_cfg.clone();
    let c = &new_cfg.theme;

    let background = c.color_from_str(&c.background);
    let foreground = c.color_from_str(&c.foreground);
    let sel = c.color_from_str(&c.selection_color);
    let font = Font::by_name(&c.font_family);

    wind.set_color(background);
    wind.redraw();
    header.set_color(background);
    header.set_label_color(foreground);
    header.set_label_font(font);
    header.set_label_size(c.font_size);
    header.redraw();

    editor.borrow_mut().set_color(background);
    editor.borrow_mut().set_text_color(foreground);
    editor.borrow_mut().set_cursor_style(c.cursor_style.to_fltk_cursor());
    editor.borrow_mut().set_cursor_color(foreground);
    editor.borrow_mut().set_selection_color(sel);

    editor.borrow_mut().set_text_font(font);
    editor.borrow_mut().set_text_size(c.font_size);

    status_label.set_color(background);
    status_label.set_label_color(foreground);
    status_label.set_label_font(font);
    status_label.set_label_size(c.font_size);

    if let Some(dot) = dot {
        refresh_status_dot(Some(dot), &cfg.borrow().theme);
    }

    if let Some(sb) = scrollbar {
        sb.set_colors(background, foreground);
        sb.set_style(cfg.borrow().theme.scrollbar_style.into());
    }

    if let Some(handle) = blink_timeout_handle.borrow_mut().take() {
        app::remove_timeout3(handle);
    }
    *blink_paused.borrow_mut() = false;
    *blink_state.borrow_mut() = true;
    editor_clone.borrow_mut().show_cursor(true);

    if c.cursor_flash {
        let interval = c.cursor_flash_interval;
        let blink_state_clone2 = blink_state.clone();
        let blink_paused_clone2 = blink_paused.clone();
        let editor_clone2 = editor_clone.clone();
        let blink_timeout_handle_clone2 = blink_timeout_handle.clone();

        let cb = Box::new(move |handle: app::TimeoutHandle| {
            if !*blink_paused_clone2.borrow() {
                let mut state = blink_state_clone2.borrow_mut();
                *state = !*state;
                let mut ed = editor_clone2.borrow_mut();
                if *state {
                    ed.show_cursor(true);
                } else {
                    ed.show_cursor(false);
                }
            } else {
                editor_clone2.borrow_mut().show_cursor(true);
                *blink_state_clone2.borrow_mut() = true;
            }
            *blink_timeout_handle_clone2.borrow_mut() = Some(handle);
            app::repeat_timeout3(interval, handle);
        });

        *blink_callback.borrow_mut() = Some(cb);

        let blink_callback_clone = blink_callback.clone();
        let handle = app::add_timeout3(interval, move |h| {
            if let Some(ref mut cb) = *blink_callback_clone.borrow_mut() {
                cb(h);
            }
        });
        *blink_timeout_handle.borrow_mut() = Some(handle);
    } else {
        editor_clone.borrow_mut().show_cursor(true);
        *blink_callback.borrow_mut() = None;
    }
}

fn get_max_top(
    editor: &TextEditor,
    scrollbar: &mut widgets::scrollbar::ScrollBar,
) -> i32 {
    let buf = editor.buffer().unwrap();
    let line_height = (editor.text_size() as f32 * 1.4) as i32;
    let visible_lines = (editor.height() / line_height).max(1);
    let total_lines = buf.count_lines(0, buf.length());
    let last_visible_line = if total_lines > 0 { total_lines - 1 } else { 0 };
    let max_top = if last_visible_line < visible_lines {
        0
    } else {
        last_visible_line.saturating_sub(visible_lines - 1)
    };

    scrollbar.set_range(0, max_top);
    if max_top > 0 {
        scrollbar.show();
    } else {
        scrollbar.hide();
    }

    max_top
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        help::print_help();
        return;
    }

    fontconfig_init::init();
    let cfg = Rc::from(RefCell::from(Config::default()));
    let app = app::App::default();
    let widget_scheme = WidgetScheme::new(SchemeType::Sweet);
    widget_scheme.apply();

    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 { args[1].clone() } else { "out.txt".to_string() };
    let abs_path = fs::canonicalize(&file_path)
        .map(|p| p.display().to_string())
        .unwrap_or(file_path.clone());
    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("untitled");

    let window_title = format!("Skrift - {}", file_name);
    let wind = Rc::new(RefCell::new(Window::new(100, 100, 800, 600, window_title.as_str())));
    let header = Rc::new(RefCell::new(Frame::new(0, 0, 800, 30, abs_path.as_str())));

    let pad = 10;
    let scrollbar_width = 12;
    let editor_height = 540;
    let editor_width = 800 - pad * 2 - scrollbar_width;
    let editor = Rc::from(RefCell::from(TextEditor::new(
        pad, 30, editor_width, editor_height, "",
    )));
    editor.borrow_mut().set_scrollbar_align(fltk::enums::Align::Inside);
    editor.borrow_mut().set_frame(fltk::enums::FrameType::NoBox);
    let top_line = Rc::new(RefCell::new(1));

    let mut scrollbar = widgets::scrollbar::ScrollBar::new(
        pad + editor_width,
        30,
        scrollbar_width,
        editor_height,
    );
    scrollbar.set_style(widgets::scrollbar::ScrollBarStyle::Rounded);

    let top_line_for_scrollbar = top_line.clone();
    let editor_for_scrollbar = editor.clone();
    scrollbar.set_on_change(move |val| {
        editor_for_scrollbar.borrow_mut().scroll(val, 0);
        *top_line_for_scrollbar.borrow_mut() = val;
    });
    scrollbar.show();
    let scrollbar = Rc::new(RefCell::new(scrollbar));

    let mut status_label = Frame::new(
        pad, 30 + editor_height, editor_width, 30, "",
    );

    let file_exists = Rc::new(RefCell::new(std::path::Path::new(&file_path).exists()));

    let status_dot = Rc::new(RefCell::new(None));
    let font_size = cfg.borrow().theme.font_size;
    let dot_x = font_size / 2;
    let dot_y = font_size / 2;
    let dot_color = cfg.borrow().theme.color_from_str(&cfg.borrow().theme.negative_color);
    let mut dot = widgets::dot::Dot::new(dot_x, dot_y, 5, dot_color);
    if *file_exists.borrow() {
        dot.hide();
    } else {
        update_status_dot(Some(&mut dot), StatusDotState::Negative, font_size, &cfg.borrow().theme);
    }
    *status_dot.borrow_mut() = Some(dot);
    let mut buf = TextBuffer::default();
    if std::path::Path::new(&file_path).exists() {
        if let Ok(contents) = fs::read_to_string(&file_path) {
            buf.set_text(&contents);
        }
    } else {
        println!("File not found: {}", file_path);
    }
    editor.borrow_mut().set_buffer(buf.clone());
    get_max_top(&editor.borrow(), &mut scrollbar.borrow_mut());

    let blink_state = Rc::from(RefCell::from(true));
    let blink_paused = Rc::from(RefCell::from(false));
    let editor_clone = editor.clone();

    let blink_timeout_handle = Rc::from(RefCell::from(None));
    fn pause_blink(
        blink_paused: Rc<RefCell<bool>>,
        editor: Rc<RefCell<TextEditor>>,
        blink_state: Rc<RefCell<bool>>,
        interval: f64,
        blink_timeout_handle: Rc<RefCell<Option<app::TimeoutHandle>>>,
        blink_callback: Rc<RefCell<Option<Box<dyn FnMut(app::TimeoutHandle)>>>>,
    ) {
        *blink_paused.borrow_mut() = true;
        *blink_state.borrow_mut() = true;
        editor.borrow_mut().show_cursor(true);

        if let Some(handle) = blink_timeout_handle.borrow_mut().take() {
            app::remove_timeout3(handle);
        }

        let blink_paused_clone = blink_paused.clone();
        let blink_timeout_handle_clone = blink_timeout_handle.clone();
        let blink_callback_clone = blink_callback.clone();

        let handle = app::add_timeout3(interval, {
            let blink_timeout_handle_clone = blink_timeout_handle_clone.clone();
            let blink_callback_clone = blink_callback_clone.clone();
            let blink_paused_clone = blink_paused_clone.clone();
            move |_h| {
                *blink_paused_clone.borrow_mut() = false;
                if let Some(ref mut cb) = *blink_callback_clone.borrow_mut() {
                    let cb_boxed = cb as *mut Box<dyn FnMut(app::TimeoutHandle)>;
                    let blink_timeout_handle_clone = blink_timeout_handle_clone.clone();
                    let new_handle = app::add_timeout3(interval, move |hh| {
                        unsafe {
                            (*cb_boxed)(hh);
                        }
                    });
                    *blink_timeout_handle_clone.borrow_mut() = Some(new_handle);
                }
            }
        });
        *blink_timeout_handle.borrow_mut() = Some(handle);
    }

    let cfg_clone = cfg.clone();
    let blink_state_clone = blink_state.clone();
    let blink_paused_clone = blink_paused.clone();
    let blink_timeout_handle_clone = blink_timeout_handle.clone();
    let editor_clone2 = editor_clone.clone();

    let blink_callback: Rc<RefCell<Option<Box<dyn FnMut(app::TimeoutHandle)>>>> =
        Rc::from(RefCell::from(None));

    if cfg_clone.borrow().theme.cursor_flash {
        let interval = cfg_clone.borrow().theme.cursor_flash_interval;
        let blink_state_clone2 = blink_state_clone.clone();
        let blink_paused_clone2 = blink_paused_clone.clone();
        let editor_clone3 = editor_clone2.clone();
        let blink_timeout_handle_clone2 = blink_timeout_handle_clone.clone();

        let cb = Box::new(move |handle: app::TimeoutHandle| {
            if !*blink_paused_clone2.borrow() {
                let mut state = blink_state_clone2.borrow_mut();
                *state = !*state;
                let mut ed = editor_clone3.borrow_mut();
                if *state {
                    ed.show_cursor(true);
                } else {
                    ed.show_cursor(false);
                }
            } else {
                editor_clone3.borrow_mut().show_cursor(true);
                *blink_state_clone2.borrow_mut() = true;
            }
            *blink_timeout_handle_clone2.borrow_mut() = Some(handle);
            app::repeat_timeout3(interval, handle);
        });

        *blink_callback.borrow_mut() = Some(cb);

        let blink_callback_clone = blink_callback.clone();
        let handle = app::add_timeout3(interval, move |h| {
            if let Some(ref mut cb) = *blink_callback_clone.borrow_mut() {
                cb(h);
            }
        });
        *blink_timeout_handle.borrow_mut() = Some(handle);
    } else {
        editor_clone.borrow_mut().show_cursor(true);
    }

    load_config_and_apply(
        &cfg,
        &editor,
        &mut wind.borrow_mut(),
        &mut header.borrow_mut(),
        blink_state.clone(),
        blink_paused.clone(),
        blink_timeout_handle.clone(),
        blink_callback.clone(),
        editor_clone.clone(),
        &mut status_label,
        status_dot.borrow_mut().as_mut(),
        Some(&mut *scrollbar.borrow_mut()),
    );

    wind.borrow_mut().resizable(&editor.borrow().as_base_widget());
    wind.borrow_mut().end();
    wind.borrow_mut().show();

    wind.borrow_mut().handle({
        let editor = editor.clone();
        let scrollbar = scrollbar.clone();
        move |_w, ev| {
            if ev == fltk::enums::Event::Resize {
                let ed = editor.borrow_mut();
                let mut sb = scrollbar.borrow_mut();
                get_max_top(&ed, &mut sb);
            }
            false
        }
    });

    let buf_rc = Rc::from(RefCell::from(buf));

    let last_cursor_pos = Rc::new(RefCell::new(-1i32));
    fn update_status_label(editor: &TextEditor, label: &mut Frame, last_pos: &Rc<RefCell<i32>>) {
        let current_pos = editor.insert_position();
        if current_pos == *last_pos.borrow() {
            return;
        }
        *last_pos.borrow_mut() = current_pos;

        if let Some(buf) = editor.buffer() {
            let text = buf.text();
            let mut line = 1;
            let mut col = 1;
            let mut count = 0;
            for c in text.chars() {
                if count == current_pos {
                    break;
                }
                if c == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                count += 1;
            }
            label.set_label(&format!("{}, {}", line, col));
        }
    }

    let status_dot_clone = status_dot.clone();
    let last_cursor_pos_clone = last_cursor_pos.clone();

    editor.borrow_mut().handle({
        let cfg = cfg.clone();
        let buf = buf_rc.clone();
        let editor = editor.clone();
        let blink_state = blink_state.clone();
        let blink_paused = blink_paused.clone();
        let interval = cfg.borrow().theme.cursor_flash_interval;
        let blink_timeout_handle = blink_timeout_handle.clone();
        let blink_callback = blink_callback.clone();
        let wind_ptr = Rc::as_ptr(&wind) as *mut RefCell<Window>;
        let header_ptr = Rc::as_ptr(&header) as *mut RefCell<Frame>;
        let editor_clone = editor_clone.clone();
        let status_label_ptr = &mut status_label as *mut Frame;
        let status_dot = status_dot_clone;
        let last_cursor_pos = last_cursor_pos_clone;
        let top_line = top_line.clone();
        let scrollbar = scrollbar.clone();

        move |_, ev| {
            match ev {
                fltk::enums::Event::KeyDown | fltk::enums::Event::KeyUp => {
                    pause_blink(
                        blink_paused.clone(),
                        editor.clone(),
                        blink_state.clone(),
                        interval,
                        blink_timeout_handle.clone(),
                        blink_callback.clone(),
                    );

                    {
                        let ed = editor.borrow_mut();
                        let mut sb = scrollbar.borrow_mut();
                        let max_top = get_max_top(&ed, &mut sb);

                        if let Some(buf) = ed.buffer() {
                            let total_lines = buf.count_lines(0, buf.length());
                            let insert_pos = ed.insert_position();
                            let line_of_cursor = buf.line_start(insert_pos);
                            let line_idx = buf.count_lines(0, line_of_cursor);
                            if line_idx >= total_lines - 1 {
                                *top_line.borrow_mut() = max_top;
                            }
                        }

                        let top = *top_line.borrow();
                        sb.set_value(top);
                    }
                    unsafe {
                        update_status_label(&editor.borrow(), &mut *status_label_ptr, &last_cursor_pos);
                    }
                }
                fltk::enums::Event::Push | fltk::enums::Event::Drag | fltk::enums::Event::Released => {
                    let editor = editor.clone();
                    let status_label_ptr = status_label_ptr.clone();
                    let last_cursor_pos = last_cursor_pos.clone();
                    unsafe {
                        update_status_label(&editor.borrow(), &mut *status_label_ptr, &last_cursor_pos);
                    }
                }
                fltk::enums::Event::MouseWheel => {
                    let mut ed = editor.borrow_mut();
                    let dy = app::event_dy();
                    let mut top = *top_line.borrow();
                    let max_top = get_max_top(&ed, &mut scrollbar.borrow_mut());

                    let scroll_multiplier = cfg.borrow().editor.scroll_multiplier.max(1);

                    match dy {
                        fltk::app::MouseWheel::Up => {
                            top = (top - scroll_multiplier).max(0);
                        }
                        fltk::app::MouseWheel::Down => {
                            top = (top + scroll_multiplier).min(max_top);
                        }
                        _ => {}
                    }

                    *top_line.borrow_mut() = top;
                    ed.scroll(top, 0);
                    scrollbar.borrow_mut().set_value(top);

                    return true;
                }
                _ => {}
            }

            if let fltk::enums::Event::KeyDown = ev {
                let bindings: Vec<(Binding, String)> = {
                    let cfg_borrow = cfg.borrow();
                    cfg_borrow.bindings.iter().map(|(a, b)| (a.clone(), b.clone())).collect()
                };
                for (binding, cmd) in bindings {
                    let matched = config::Config::shortcut_matches(&cmd);
                    if matched {
                        println!("Shortcut matched: {:?} -> {}", binding, cmd);
                        match binding {
                            Binding::Save => {
                                if let Some(parent) = std::path::Path::new(&file_path).parent() {
                                    if !parent.exists() {
                                        std::fs::create_dir_all(parent).expect("Failed to create parent directory");
                                    }
                                }
                                std::fs::write(&file_path, buf.borrow().text()).expect("write failed");
                                if !*file_exists.borrow() {
                                    if let Some(mut dot) = status_dot.borrow_mut().take() {
                                        update_status_dot(Some(&mut dot), StatusDotState::Hidden, cfg.borrow().theme.font_size, &cfg.borrow().theme);
                                    }
                                }
                                if let Some(dot) = status_dot.borrow_mut().as_mut() {
                                    show_status_dot_timed(
                                        Some(dot),
                                        StatusDotState::Positive,
                                        cfg.borrow().theme.font_size,
                                        &cfg.borrow().theme,
                                        1.0,
                                        status_dot.clone(),
                                    );
                                }
                            }
                            Binding::Quit => {
                                println!("Quitting app");
                                app::quit()
                            },
                            Binding::Reload => {
                                unsafe {
                                    load_config_and_apply(
                                        &cfg,
                                        &editor,
                                        &mut (*wind_ptr).get_mut(),
                                        &mut (*header_ptr).get_mut(),
                                        blink_state.clone(),
                                        blink_paused.clone(),
                                        blink_timeout_handle.clone(),
                                        blink_callback.clone(),
                                        editor_clone.clone(),
                                        &mut *status_label_ptr,
                                        status_dot.borrow_mut().as_mut(),
                                        Some(&mut *scrollbar.borrow_mut()),
                                    );
                                }
                                println!("Config reloaded");
                            }
                            Binding::MoveLineUp => {
                                let mut ed = editor.borrow_mut();
                                if let Some(mut buf) = ed.buffer().map(|b| b.clone()) {
                                    let pos = ed.insert_position() as usize;
                                    let text = buf.text();
                                    let lines: Vec<&str> = text.split('\n').collect();

                                    let mut char_count = 0;
                                    let mut curr_idx = 0;
                                    let mut col = 0;
                                    for (i, line) in lines.iter().enumerate() {
                                        if pos < char_count + line.len() + 1 {
                                            curr_idx = i;
                                            col = pos - char_count;
                                            break;
                                        }
                                        char_count += line.len() + 1;
                                    }

                                    if curr_idx > 0 {
                                        let mut new_lines = lines.clone();
                                        new_lines.swap(curr_idx, curr_idx - 1);
                                        buf.set_text(&new_lines.join("\n"));
                                        let mut new_char_count = 0;
                                        for i in 0..curr_idx - 1 {
                                            new_char_count += new_lines[i].len() + 1;
                                        }
                                        let new_pos = new_char_count + col.min(new_lines[curr_idx - 1].len());
                                        ed.set_insert_position(new_pos as i32);
                                    }
                                }
                            },
                            Binding::MoveLineDown => {
                                let mut ed = editor.borrow_mut();
                                if let Some(mut buf) = ed.buffer().map(|b| b.clone()) {
                                    let pos = ed.insert_position() as usize;
                                    let text = buf.text();
                                    let lines: Vec<&str> = text.split('\n').collect();

                                    let mut char_count = 0;
                                    let mut curr_idx = 0;
                                    let mut col = 0;
                                    for (i, line) in lines.iter().enumerate() {
                                        if pos < char_count + line.len() + 1 {
                                            curr_idx = i;
                                            col = pos - char_count;
                                            break;
                                        }
                                        char_count += line.len() + 1;
                                    }

                                    if curr_idx + 1 < lines.len() {
                                        let mut new_lines = lines.clone();
                                        new_lines.swap(curr_idx, curr_idx + 1);
                                        buf.set_text(&new_lines.join("\n"));
                                        let mut new_char_count = 0;
                                        for i in 0..curr_idx + 1 {
                                            new_char_count += new_lines[i].len() + 1;
                                        }
                                        let new_pos = new_char_count + col.min(new_lines[curr_idx + 1].len());
                                        ed.set_insert_position(new_pos as i32);
                                    }
                                }
                            },
                        }
                        return true;
                    }
                }
                return false;
            }
            false
        }
    });

    update_status_label(&editor.borrow(), &mut status_label, &last_cursor_pos);

    let config_path = {
        let mut path = dirs::home_dir().unwrap_or(std::path::PathBuf::from("."));
        path.push(".config/skrift/config.skrift");
        path
    };
    let (tx, rx) = channel();
    let _watcher = config_watcher::start_config_watcher(config_path, tx);

    let cfg_ptr = cfg.clone();
    let editor_ptr = editor.clone();
    let blink_state_ptr = blink_state.clone();
    let blink_paused_ptr = blink_paused.clone();
    let blink_timeout_handle_ptr = blink_timeout_handle.clone();
    let blink_callback_ptr = blink_callback.clone();
    let editor_clone_ptr = editor_clone.clone();
    let status_label_ptr = &mut status_label as *mut Frame;
    let status_dot_ptr = status_dot.clone();
    let wind_ptr = wind.clone();
    let header_ptr = header.clone();
    let wind_ptr_cb = wind_ptr.clone();
    let header_ptr_cb = header_ptr.clone();

    let config_check_interval = 0.1;
    let config_rx = Rc::new(RefCell::new(rx));

    app::add_timeout3(config_check_interval, {
        let config_rx = config_rx.clone();
        let cfg_ptr = cfg_ptr.clone();
        let editor_ptr = editor_ptr.clone();
        let blink_state_ptr = blink_state_ptr.clone();
        let blink_paused_ptr = blink_paused_ptr.clone();
        let blink_timeout_handle_ptr = blink_timeout_handle_ptr.clone();
        let blink_callback_ptr = blink_callback_ptr.clone();
        let editor_clone_ptr = editor_clone_ptr.clone();
        let status_dot_ptr = status_dot_ptr.clone();
        let wind_ptr_cb = wind_ptr_cb.clone();
        let header_ptr_cb = header_ptr_cb.clone();

        move |handle| {
            let mut config_changed = false;
            while let Ok(()) = config_rx.borrow().try_recv() {
                config_changed = true;
            }

            if config_changed {
                unsafe {
                    load_config_and_apply(
                        &cfg_ptr,
                        &editor_ptr,
                        &mut wind_ptr_cb.borrow_mut(),
                        &mut header_ptr_cb.borrow_mut(),
                        blink_state_ptr.clone(),
                        blink_paused_ptr.clone(),
                        blink_timeout_handle_ptr.clone(),
                        blink_callback_ptr.clone(),
                        editor_clone_ptr.clone(),
                        &mut *status_label_ptr,
                        status_dot_ptr.borrow_mut().as_mut(),
                        Some(&mut *scrollbar.borrow_mut()),
                    );
                }
            }

            app::repeat_timeout3(config_check_interval, handle);
        }
    });

    app.run().unwrap();
}
