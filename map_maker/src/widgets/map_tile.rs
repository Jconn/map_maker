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
    layout, mouse, Background, Clipboard, Color, Element, Event, Hasher, Layout, Length, Point,
    Rectangle, Size, Vector, Widget,
};

pub struct MapTile {
    handle: image::Handle,
    width: Length,
    height: Length,
}

impl MapTile {
    pub fn new(bytes: Bytes) -> Self {
        let handle = image::Handle::from_memory(bytes.to_vec());
        Self {
            handle,
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    /// Returns the bounds of the underlying image, given the bounds of
    /// the [`Viewer`]. Scaling will be applied and original aspect ratio
    /// will be respected.
    fn image_size<Renderer>(&self, renderer: &Renderer, bounds: Size) -> Size
    where
        Renderer: self::Renderer + iced_native::image::Renderer + iced_native::Renderer,
    {
        let (width, height) = renderer.dimensions(&self.handle);

        let (width, height) = {
            let dimensions = (width as f32, height as f32);

            let width_ratio = bounds.width / dimensions.0;
            let height_ratio = bounds.height / dimensions.1;

            let ratio = width_ratio.min(height_ratio);

            //let scale = self.state.scale;
            let scale = 1.0;

            if ratio < 1.0 {
                (dimensions.0 * ratio * scale, dimensions.1 * ratio * scale)
            } else {
                (dimensions.0 * scale, dimensions.1 * scale)
            }
        };

        Size::new(width, height)
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for MapTile
where
    Renderer: self::Renderer + iced_native::image::Renderer + iced_native::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let (width, height) = renderer.dimensions(&self.handle);

        let mut size = limits
            .width(self.width)
            .height(self.height)
            .resolve(Size::new(width as f32, height as f32));

        let expansion_size = if height > width {
            self.width
        } else {
            self.height
        };

        // Only calculate viewport sizes if the images are constrained to a limited space.
        // If they are Fill|Portion let them expand within their alotted space.
        match expansion_size {
            Length::Shrink | Length::Units(_) => {
                let aspect_ratio = width as f32 / height as f32;
                let viewport_aspect_ratio = size.width / size.height;
                if viewport_aspect_ratio > aspect_ratio {
                    size.width = width as f32 * size.height / height as f32;
                } else {
                    size.height = height as f32 * size.width / width as f32;
                }
            }
            Length::Fill | Length::FillPortion(_) => {}
        }

        layout::Node::new(size)
    }
    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _messages: &mut Vec<Message>,
    ) -> event::Status {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

        match event {
            _ => event::Status::Ignored,
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        //renderer.draw(self.handle.clone(), layout)
        //iced_native::image::Renderer::draw(&mut renderer, self.handle.clone(), layout)

        let bounds = layout.bounds();

        let image_size = self.image_size(renderer, bounds.size());

        let translation = {
            let image_top_left = Vector::new(
                bounds.width / 2.0 - image_size.width / 2.0,
                bounds.height / 2.0 - image_size.height / 2.0,
            );
            image_top_left
        };
        self::Renderer::draw(
            renderer,
            bounds,
            image_size,
            translation,
            self.handle.clone(),
        )
        //renderer.draw(self.handle.clone(), layout)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;
        self.handle.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
}

/// The local state of a [`Viewer`].
#[derive(Debug, Clone, Copy)]
pub struct State {
    scale: f32,
    starting_offset: Vector,
    current_offset: Vector,
    cursor_grabbed_at: Option<Point>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            scale: 1.0,
            starting_offset: Vector::default(),
            current_offset: Vector::default(),
            cursor_grabbed_at: None,
        }
    }
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> Self {
        State::default()
    }

    /// Returns the current offset of the [`State`], given the bounds
    /// of the [`Viewer`] and its image.
    fn offset(&self, bounds: Rectangle, image_size: Size) -> Vector {
        let hidden_width = (image_size.width - bounds.width / 2.0).max(0.0).round();

        let hidden_height = (image_size.height - bounds.height / 2.0).max(0.0).round();

        Vector::new(
            self.current_offset.x.min(hidden_width).max(-hidden_width),
            self.current_offset.y.min(hidden_height).max(-hidden_height),
        )
    }

    /// Returns if the cursor is currently grabbed by the [`Viewer`].
    pub fn is_cursor_grabbed(&self) -> bool {
        self.cursor_grabbed_at.is_some()
    }
}

/// The renderer of an [`Viewer`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Viewer`] in your user interface.
///
/// [renderer]: crate::renderer
pub trait Renderer: iced_native::Renderer + iced_native::image::Renderer + Sized {
    /// Draws the [`Viewer`].
    ///
    /// It receives:
    /// - the [`State`] of the [`Viewer`]
    /// - the bounds of the [`Viewer`] widget
    /// - the [`Size`] of the scaled [`Viewer`] image
    /// - the translation of the clipped image
    /// - the [`Handle`] to the underlying image
    /// - whether the mouse is over the [`Viewer`] or not
    ///
    /// [`Handle`]: image::Handle
    fn draw(
        &mut self,
        bounds: Rectangle,
        image_size: Size,
        translation: Vector,
        handle: image::Handle,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<MapTile> for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer + iced_native::image::Renderer,
    Message: 'a,
{
    fn from(map_tile: MapTile) -> Element<'a, Message, Renderer> {
        Element::new(map_tile)
    }
}

impl<B> Renderer for iced_graphics::Renderer<B>
where
    B: Backend + backend::Image,
{
    fn draw(
        &mut self,
        bounds: Rectangle,
        image_size: Size,
        translation: Vector,
        handle: image::Handle,
    ) -> Self::Output {
        (
            {
                Primitive::Clip {
                    bounds,
                    content: Box::new(Primitive::Translate {
                        translation,
                        content: Box::new(Primitive::Image {
                            handle,
                            bounds: Rectangle {
                                x: bounds.x,
                                y: bounds.y,
                                ..Rectangle::with_size(image_size)
                            },
                        }),
                    }),
                    offset: Vector::new(0, 0),
                }
            },
            { mouse::Interaction::Idle },
        )
    }
}
