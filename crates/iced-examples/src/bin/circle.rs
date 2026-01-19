// Circle example for iced - custom widget
use iced::advanced::renderer;

use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::mouse::Cursor;
use iced::advanced::widget::{Tree, Widget};
use iced::widget::{center, column, slider, text};
use iced::{Center, Color, Element, Length, Rectangle, Size, border};

fn main() -> iced::Result {
    iced::run(Example::update, Example::view)
}

struct Example {
    radius: f32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    RadiusChanged(f32),
}

impl Example {
    fn new() -> Self {
        Self { radius: 50.0 }
    }

    fn update(&mut self, msg: Message) {
        match msg {
            Message::RadiusChanged(radius) => {
                self.radius = radius;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = column![
            circle(self.radius),
            text(format!("Radius: {:.2}", self.radius)),
            slider(1.0..=100.0, self.radius, Message::RadiusChanged).step(0.01)
        ]
        .padding(20)
        .spacing(20)
        .max_width(500)
        .align_x(Center);

        center(content).into()
    }
}

impl Default for Example {
    fn default() -> Self {
        Self::new()
    }
}

pub fn circle(radius: f32) -> Circle {
    Circle::new(radius)
}

pub struct Circle {
    radius: f32,
}

impl Circle {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Circle
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &Renderer, _limits: &Limits) -> Node {
        Node::new(Size::new(self.radius * 2.0, self.radius * 2.0))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: border::rounded(self.radius), // this line is the critical part
                ..renderer::Quad::default()
            },
            Color::from_rgba(1.0, 1.0, 0.0, 1.0),
        );
    }
}

impl<Message, Theme, Renderer> From<Circle> for Element<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(circle: Circle) -> Self {
        Self::new(circle)
    }
}
