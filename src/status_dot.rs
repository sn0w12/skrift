use crate::config;
use crate::widgets::dot::Dot;
use fltk::enums::Color;

#[derive(Clone)]
pub enum StatusDotState {
    Hidden,
    Negative,
    Positive,
}

pub fn update_status_dot(dot: Option<&mut Dot>, state: StatusDotState, font_size: i32, theme: &config::Theme) {
    if let Some(dot) = dot {
        dot.current_state = state.clone();
        let dot_x = font_size / 2 + 2;
        let dot_y = font_size / 2 + 2;
        dot.set_position(dot_x, dot_y);
        dot.set_radius(font_size / 4);
        match state {
            StatusDotState::Hidden => dot.hide(),
            StatusDotState::Negative => {
                dot.set_color(theme.color_from_str(&theme.negative_color));
                dot.show();
            }
            StatusDotState::Positive => {
                dot.set_color(Color::Green);
                dot.show();
            }
        }
    }
}

pub fn show_status_dot_timed(
    dot: Option<&mut Dot>,
    state: StatusDotState,
    font_size: i32,
    theme: &config::Theme,
    duration: f64,
    status_dot_rc: std::rc::Rc<std::cell::RefCell<Option<Dot>>>,
) {
    let prev_state = dot.as_ref().map(|d| d.current_state.clone());
    update_status_dot(dot, state, font_size, theme);
    let theme = theme.clone();
    fltk::app::add_timeout3(duration, move |_handle| {
        if let Some(dot) = status_dot_rc.borrow_mut().as_mut() {
            if let Some(state) = prev_state.clone() {
                update_status_dot(Some(dot), state, font_size, &theme);
            } else {
                update_status_dot(Some(dot), StatusDotState::Hidden, font_size, &theme);
            }
        }
    });
}

pub fn refresh_status_dot(dot: Option<&mut Dot>, c: &config::Theme) {
    if let Some(dot) = dot {
        let state = dot.current_state.clone();
        update_status_dot(Some(dot), state, c.font_size, c);
    }
}