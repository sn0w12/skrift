use fltk::{widget::Widget, prelude::*, enums::Color, draw};

pub fn draw_filled_circle(x: i32, y: i32, radius: i32, color: Color) {
    draw::set_draw_color(color);
    draw::draw_pie(
        (x - radius) as i32,
        (y - radius) as i32,
        (radius * 2) as i32,
        (radius * 2) as i32,
        0.0,
        360.0,
    );
}

pub struct Dot {
    pub widget: Widget,
    x: i32,
    y: i32,
    radius: i32,
    color: Color,
    pub current_state: crate::status_dot::StatusDotState,
}

impl Dot {
    pub fn new(x: i32, y: i32, radius: i32, color: Color) -> Self {
        let widget = Widget::new(x, y, radius * 2, radius * 2, "");
        let mut dot = Dot {
            widget,
            x,
            y,
            radius,
            color,
            current_state: crate::status_dot::StatusDotState::Hidden,
        };
        dot.update_draw();
        dot
    }

    fn update_draw(&mut self) {
        let radius = self.radius;
        let color = self.color;
        self.widget.draw({
            let x = self.x;
            let y = self.y;
            move |_w| {
                draw_filled_circle(
                    x + radius,
                    y + radius,
                    radius,
                    color,
                );
            }
        });
        self.widget.redraw();
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
        self.widget.set_pos(x, y);
        self.update_draw();
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
        self.update_draw();
    }

    pub fn set_radius(&mut self, radius: i32) {
        self.radius = radius;
        self.widget.resize(self.x, self.y, radius * 2, radius * 2);
        self.update_draw();
    }

    pub fn hide(&mut self) {
        self.widget.hide();
    }

    pub fn show(&mut self) {
        self.widget.show();
    }
}