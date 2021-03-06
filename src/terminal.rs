use std::io::{self, Write, BufWriter};

use crossterm as ct;
use super::spatial::Vec2;

// ================================================================================
// VISUAL ELEMENT
// ================================================================================
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Color {
    Black,
    White,
    Grey,
    DarkGrey,
    LightGrey,
    Red,
    Green,
    Blue,
    Cyan,
    Yellow,
    Magenta,
    Xterm(u8),
}

impl Color {
    pub fn code(&self) -> u8 {
        match *self {
            Color::Black => 16,
            Color::White => 231,
            Color::Grey => 244,
            Color::DarkGrey => 238,
            Color::LightGrey => 250,
            Color::Red => 196,
            Color::Green => 46,
            Color::Blue => 21,
            Color::Cyan => 51,
            Color::Yellow => 226,
            Color::Magenta => 201,
            Color::Xterm(code) => code,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Style {
    Plain,
    Bold,
}

/*
fn style_impl(style: Style) -> ct::style::Attribute {
    match style {
        Style::Plain => ct::style::Attribute::NoBold,
        Style::Bold => ct::style::Attribute::Bold,
    }
}
*/

#[derive(Clone, Copy)]
pub struct VisualElement {
    pub style: Style,
    pub background: Color,
    pub foreground: Color,
    pub value: char,
}

impl VisualElement {
    pub fn new() -> VisualElement {
        VisualElement {
            style: Style::Plain,
            background: Color::Black,
            foreground: Color::White,
            value: ' ',
        }
    }
}

// ================================================================================
// CANVAS
// ================================================================================
pub struct Canvas {
    data: Vec<VisualElement>,
    dimension: Vec2,
    default_element: VisualElement,
}

impl Canvas {
    pub fn new(dimension: Vec2, default_element: &VisualElement) -> Canvas {
        let mut data = Vec::new();
        data.resize((dimension.x * dimension.y) as usize, *default_element);
        Canvas {
            data,
            dimension,
            default_element: *default_element,
        }
    }

    pub fn default_element(&self) -> &VisualElement {
        &self.default_element
    }

    pub fn set_default_element(&mut self, element: &VisualElement) {
        self.default_element = *element;
    }

    pub fn dimension(&self) -> Vec2 {
        self.dimension
    }

    pub fn contains(&self, pos: Vec2) -> bool {
        0 <= pos.x && 0 <= pos.y &&
        pos.x < self.dimension.x &&
        pos.y < self.dimension.y
    }

    pub fn elem(&self, pos: Vec2) -> Option<&VisualElement> {
        if self.contains(pos) {
            Some(&self.data[(pos.y * self.dimension.x + pos.x) as usize])
        }
        else { None }
    }

    pub fn elem_mut(&mut self, pos: Vec2) -> Option<&mut VisualElement> {
        if self.contains(pos) {
            Some(&mut self.data[(pos.y * self.dimension.x + pos.x) as usize])
        }
        else { None }
    }

    pub fn clear(&mut self) {
        self.fill(&self.default_element().clone());
    }

    pub fn fill(&mut self, elem: &VisualElement) {
        self.data.iter_mut().map(|x| *x = *elem).count();
    }

    pub fn data(&self) -> &Vec<VisualElement> {
        &self.data
    }
}

// ================================================================================
// WINDOW
// ================================================================================
pub struct Window {
    canvas: Canvas,
    target: BufWriter<io::Stdout>,
}

impl Window {
    pub fn new() -> Window {
        Window {
            canvas: Canvas::new(size(), &VisualElement::new()),
            target: BufWriter::with_capacity(size().x as usize * size().y as usize * 50, io::stdout()),
        }
    }

    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    pub fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    pub fn size(&self) -> Vec2 {
        self.canvas.dimension()
    }

    pub fn open(&mut self) {
        ct::queue!(self.target, ct::terminal::EnterAlternateScreen).unwrap();
        ct::queue!(self.target, ct::style::ResetColor).unwrap();
        ct::queue!(self.target, ct::style::SetAttribute(ct::style::Attribute::Reset)).unwrap();
        ct::queue!(self.target, ct::cursor::Hide).unwrap();

        self.clean_state();
        self.raw_mode(true);

        self.target.flush().unwrap();
    }

    pub fn raw_mode(&mut self, enable: bool) {
        if enable {
            ct::terminal::enable_raw_mode().unwrap();
        } else {
            ct::terminal::disable_raw_mode().unwrap();
        }
    }

    pub fn close(&mut self) {
        self.raw_mode(false);

        ct::queue!(self.target, ct::cursor::Show).unwrap();
        ct::queue!(self.target, ct::style::SetAttribute(ct::style::Attribute::Reset)).unwrap();
        ct::queue!(self.target, ct::style::ResetColor).unwrap();
        ct::queue!(self.target, ct::terminal::LeaveAlternateScreen).unwrap();

        self.target.flush().unwrap();
    }

    pub fn clear(&mut self) {
        if self.canvas.dimension() != size() {
            self.canvas = Canvas::new(size(), self.canvas.default_element());
        }
        else {
            self.canvas.fill(&self.canvas.default_element().clone());
        }
    }

    pub fn draw(&mut self) {
        self.clean_state();
        let mut last_foreground = self.canvas.default_element().foreground;
        let mut last_background = self.canvas.default_element().background;
        //let mut last_style = self.canvas.default_element().style;
        let target = &mut self.target;
        
        for element in self.canvas.data().iter() {
            /*
            if last_style != element.style {
                let term_attribute = style_impl(element.style);
                ct::queue!(self.target, ct::style::SetAttribute(term_attribute)).unwrap();
                last_style = element.style
            }
            */
            if last_foreground != element.foreground {
                let term_color = ct::style::Color::AnsiValue(element.foreground.code());
                ct::queue!(target, ct::style::SetForegroundColor(term_color)).unwrap();
                last_foreground = element.foreground
            }
            if last_background != element.background {
                let term_color = ct::style::Color::AnsiValue(element.background.code());
                ct::queue!(target, ct::style::SetBackgroundColor(term_color)).unwrap();
                last_background = element.background
            }
            ct::queue!(target, ct::style::Print(element.value)).unwrap();
        }
        self.clean_state();
        self.target.flush().unwrap();
    }

    fn clean_state(&mut self) {
        //ct::queue!(self.target, ct::style::SetAttribute(ct::style::Attribute::NoBold)).unwrap();

        let term_foreground = ct::style::Color::AnsiValue(self.canvas.default_element().foreground.code());
        ct::queue!(self.target, ct::style::SetForegroundColor(term_foreground)).unwrap();

        let term_background = ct::style::Color::AnsiValue(self.canvas.default_element().background.code());
        ct::queue!(self.target, ct::style::SetBackgroundColor(term_background)).unwrap();

        ct::queue!(self.target, ct::cursor::MoveTo(0, 0)).unwrap();
    }
}

pub fn size() -> Vec2 {
    let (x, y) = ct::terminal::size().unwrap();
    Vec2::xy(x, y)
}

