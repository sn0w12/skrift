use fltk::{app, prelude::*, widget::Widget, enums::Color, draw};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScrollBarStyle {
    Boxy,
    Rounded,
}

pub struct ScrollBar {
    pub widget: Widget,
    pub min: i32,
    pub max: i32,
    pub value: i32,
    pub bg_color: Color,
    pub thumb_color: Color,
    pub style: ScrollBarStyle,
    dragging: bool,
    drag_offset: i32,
    pub on_change: Option<Box<dyn Fn(i32)>>,
}

impl ScrollBar {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let widget = Widget::new(x, y, w, h, "");
        let mut sb = ScrollBar {
            widget,
            min: 0,
            max: 100,
            value: 0,
            bg_color: Color::from_u32(0x222222),
            thumb_color: Color::from_u32(0x888888),
            style: ScrollBarStyle::Boxy,
            dragging: false,
            drag_offset: 0,
            on_change: None,
        };
        sb.update_draw();
        sb
    }

    pub fn set_colors(&mut self, bg: Color, thumb: Color) {
        self.bg_color = bg;
        self.thumb_color = thumb;
        self.update_draw();
    }

    pub fn set_style(&mut self, style: ScrollBarStyle) {
        self.style = style;
        self.update_draw();
    }

    pub fn set_range(&mut self, min: i32, max: i32) {
        self.min = min;
        self.max = max;
        self.update_draw();
    }

    pub fn set_value(&mut self, value: i32) {
        self.value = value.clamp(self.min, self.max);
        self.update_draw();
    }

    fn update_draw(&mut self) {
        let min = self.min;
        let max = self.max;
        let value = self.value;
        let w = self.widget.width();
        let h = self.widget.height();
        let bg_color = self.bg_color;
        let thumb_color = self.thumb_color;
        let style = self.style.clone();

        self.widget.draw(move |wgt| {
            draw::set_draw_color(bg_color);
            draw::draw_rectf(wgt.x(), wgt.y(), w, h);

            let thumb_h = (h as f32 * 0.2).max(20.0) as i32;
            let range = (max - min).max(1);
            let y_offset = ((h - thumb_h) as f32 * (value - min) as f32 / range as f32) as i32;
            draw::set_draw_color(thumb_color);

            draw::push_clip(wgt.x(), wgt.y(), w, h);

            match style {
                ScrollBarStyle::Boxy => {
                    draw::draw_rectf(wgt.x(), wgt.y() + y_offset, w, thumb_h);
                }
                ScrollBarStyle::Rounded => {
                    let radius = (w.min(thumb_h) / 2).min(12);
                    draw::draw_rbox(wgt.x(), wgt.y() + y_offset, w, thumb_h, radius, true, thumb_color);
                }
            }

            draw::pop_clip();
        });

        let sb_ptr = self as *mut ScrollBar;
        self.widget.handle(move |w, ev| {
            let sb = unsafe { &mut *sb_ptr };
            let mx = app::event_x() - w.x();
            let my = app::event_y() - w.y();
            let width = w.width();
            let height = w.height();
            let thumb_h = (height as f32 * 0.2).max(20.0) as i32;
            let range = (sb.max - sb.min).max(1);
            let thumb_y = ((height - thumb_h) as f32 * (sb.value - sb.min) as f32 / range as f32) as i32;

            match ev {
                fltk::enums::Event::Push => {
                    if mx >= 0 && mx < width && my >= thumb_y && my < thumb_y + thumb_h {
                        sb.dragging = true;
                        sb.drag_offset = my - thumb_y;
                        true
                    } else {
                        false
                    }
                }
                fltk::enums::Event::Drag => {
                    if sb.dragging {
                        let mut new_thumb_y = my - sb.drag_offset;
                        new_thumb_y = new_thumb_y.clamp(0, h - thumb_h);
                        let new_value = sb.min + ((new_thumb_y as f32 / (h - thumb_h) as f32) * range as f32).round() as i32;
                        if new_value != sb.value {
                            sb.value = new_value.clamp(sb.min, sb.max);
                            sb.update_draw();
                            if let Some(ref cb) = sb.on_change {
                                cb(sb.value);
                            }
                        }
                        true
                    } else {
                        false
                    }
                }
                fltk::enums::Event::Released => {
                    sb.dragging = false;
                    false
                }
                _ => false,
            }
        });
        self.widget.redraw();
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.widget.set_pos(x, y);
        self.update_draw();
    }

    pub fn set_size(&mut self, w: i32, h: i32) {
        self.widget.resize(self.widget.x(), self.widget.y(), w, h);
        self.update_draw();
    }

    pub fn show(&mut self) {
        self.widget.show();
    }

    pub fn hide(&mut self) {
        self.widget.hide();
    }

    pub fn set_on_change<F: 'static + Fn(i32)>(&mut self, cb: F) {
        self.on_change = Some(Box::new(cb));
    }
}