use bytes::{BufMut, Bytes, BytesMut};
// For now, to implement a custom native widget you will need to add
// `iced_native` and `iced_wgpu` to your dependencies.
//
// Then, you simply need to define your widget type and implement the
// `iced_native::Widget` trait with the `iced_wgpu::Renderer`.
//
// Of course, you can choose to make the implementation renderer-agnostic,
// if you wish to, by creating your own `Renderer` trait, which could be
// implemented by `iced_wgpu` and other renderers.
use iced::image;
use iced_graphics::backend::{self, Backend};
use iced_graphics::{Defaults, Primitive};
use iced_native::event;
use iced_native::{
    button, layout, mouse, overlay,layout::Limits, Background, Button, Clipboard, Color, Element, Event, Hasher,
    Layout, Length, Overlay, Point, Rectangle, Size, Text, Vector, Widget,
};

pub struct TileOverlay<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + crate::widgets::map_tile::Renderer + iced_native::button::Renderer,
{
    /// # type Button<'a, Message> =
    /// #     iced_native::Button<'a, Message, iced_native::renderer::Null>;
    zoom_in: Button<'a, Message, Renderer>,
}
impl<'a, Message, Renderer> TileOverlay<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + crate::widgets::map_tile::Renderer + iced_native::button::Renderer + iced_native::text::Renderer,
{
    pub fn new(zoom_in: Button<'a, Message, Renderer>) -> Self {
        Self {
            zoom_in,
        }
    }
    pub fn overlay(self, position: Point) -> overlay::Element<'a, Message, Renderer> {
        overlay::Element::new(position, Box::new(self))
    }
}
impl<'a, Message, Renderer> Overlay<Message, Renderer> for TileOverlay<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: crate::widgets::map_tile::Renderer + iced_native::button::Renderer,
{
    fn layout(&self, renderer: &Renderer, bounds: Size, position: Point) -> layout::Node {
        let limits = Limits::new(Size::ZERO, bounds);
        let button_layout = self.zoom_in.layout(renderer, &limits);
        button_layout
    }

    fn hash_layout(&self, state: &mut Hasher, position: Point) {
        use std::hash::Hash;

        //(self.width).hash(state);
        //(self.height).hash(state);
        self.zoom_in.hash_layout(state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> event::Status {
        self.zoom_in.on_event(
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            messages,
        )
    }
    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        self.zoom_in.draw(
            renderer,
            defaults,
            layout,
            cursor_position,
            &Rectangle::default(),
        )
    }
}
