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
    button, layout, mouse, overlay, Background, Button, Clipboard, Color, Element, Event, Hasher,
    Layout, Length, Overlay, Point, Rectangle, Size, Text, Vector, Widget,
};

pub struct MapTile<'a, B, Message, Renderer>
where
    B: Fn(&'a mut button::State) -> Button<'_, Message, Renderer>,
    Message: Clone,
    Renderer: button::Renderer,
{
    state: &'a mut button::State,
    zoom_in: B,
    tile_handles: [[Option<image::Handle>; 4]; 4],
    width: Length,
    height: Length,
}

pub struct TileOverlay<'a, B, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + self::Renderer + iced_native::button::Renderer,
    B: Fn(&'a mut button::State) -> Button<'a, Message, Renderer>,
{
    /// # type Button<'a, Message> =
    /// #     iced_native::Button<'a, Message, iced_native::renderer::Null>;
    state: &'a mut button::State,
    zoom_in: B,
    width: f32,
    height: f32,
}
impl<'a, B, Message, Renderer> TileOverlay<'a, B, Message, Renderer>
where
    Message: 'a + Clone,
    B: Fn(&mut button::State) -> Button<'_, Message, Renderer>,
    Renderer: 'a +self::Renderer + iced_native::button::Renderer + iced_native::text::Renderer,
{
    pub fn new(state: &'a mut button::State, zoom_in: B) -> Self {
        Self {
            state,
            zoom_in,
            width: 64.0,
            height: 64.0,
        }
    }
    pub fn overlay(self, position: Point) -> overlay::Element<'a, Message, Renderer> {
        overlay::Element::new(position, Box::new(self))
    }
}
impl<'a, B, Message, Renderer> Overlay<Message, Renderer> for TileOverlay<'a, B, Message, Renderer>
where
    B: Fn(&mut button::State) -> Button<'_, Message, Renderer>,
    Message: Clone,
    Renderer: self::Renderer + iced_native::button::Renderer,
{
    fn layout(&self, renderer: &Renderer, bounds: Size, position: Point) -> layout::Node {
        let size = Size::new(self.width, self.height);

        let mut node = layout::Node::new(size);
        node.move_to(position);

        node
    }

    fn hash_layout(&self, state: &mut Hasher, position: Point) {
        use std::hash::Hash;

        //(self.width).hash(state);
        //(self.height).hash(state);
        self.zoom_in.hash_layout(state);
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
impl<'a, B, Message, Renderer> MapTile<'a, B, Message, Renderer>
where
    Message: Clone,
    Renderer: self::Renderer + iced_native::button::Renderer,
    B: Fn(&'a mut button::State) -> Button<'a, Message, Renderer>,
{
    //pub fn new(state: &'a mut button::State, zoom_in: B) -> Self {
    pub fn new(tiles: [[Vec<u8>; 4]; 4], state: &'a mut button::State, zoom_in: B) -> Self {
        //let mut tile_handles: [[Option<image::Handle>; 4]; 4] = [[None; 4]; 4];
        let mut tile_handles: [[Option<image::Handle>; 4]; 4] = Default::default();

        for (idx_x, x) in tiles.iter().enumerate() {
            for (idx_y, y) in x.iter().enumerate() {
                tile_handles[idx_x][idx_y] =
                    Some(image::Handle::from_memory(tiles[idx_x][idx_y].clone()));
            }
        }

        //let tile_handles = image::Handle::from_memory(bytes.to_vec());
        Self {
            state,
            zoom_in,
            tile_handles,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    // Returns the bounds of the underlying image, given the bounds of
    // the [`Viewer`]. Scaling will be applied and original aspect ratio
    // will be respected.
    //fn image_size<Renderer>(&self, renderer: &Renderer, bounds: Size) -> Size
    //where
    //    Renderer: self::Renderer + iced_native::image::Renderer + iced_native::Renderer,
    //{
    //    //let (width, height) = renderer.dimensions(&self.handle);
    //    let (width, height) = (256 * 16, 256 * 16);

    //    let (width, height) = {
    //        let dimensions = (width as f32, height as f32);

    //        let width_ratio = bounds.width / dimensions.0;
    //        let height_ratio = bounds.height / dimensions.1;

    //        let ratio = width_ratio.min(height_ratio);

    //        //let scale = self.state.scale;
    //        let scale = 1.0;

    //        if ratio < 1.0 {
    //            (dimensions.0 * ratio * scale, dimensions.1 * ratio * scale)
    //        } else {
    //            (dimensions.0 * scale, dimensions.1 * scale)
    //        }
    //    };

    //    Size::new(width, height)
    //}
}

impl<'a, B, Message, Renderer> Widget<Message, Renderer> for MapTile<'a, B, Message, Renderer>
where
    B: 'a + Fn(&mut button::State) -> Button<'_, Message, Renderer>,
    Message: 'a + Clone,
    Renderer: 'a
        + self::Renderer
        + iced_native::image::Renderer
        + iced_native::Renderer
        + iced_native::text::Renderer
        + iced_native::button::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        //let (width, height) = renderer.dimensions(&self.handle);
        let (width, height) = (256 * 16, 256 * 16);

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

        //let image_size = self.image_size(renderer, bounds.size());

        //let translation = {
        //    let image_top_left = Vector::new(
        //        bounds.width / 2.0 - image_size.width / 2.0,
        //        bounds.height / 2.0 - image_size.height / 2.0,
        //    );
        //    image_top_left
        //};

        let translation = {
            let image_top_left = Vector::new(0.0, 0.0);
            image_top_left
        };
        self::Renderer::draw(renderer, bounds, translation, &self.tile_handles)
        //renderer.draw(self.handle.clone(), layout)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;
        self.tile_handles.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }

    /// Returns the overlay of the [`Widget`], if there is any.
    fn overlay(&mut self, _layout: Layout<'_>) -> Option<overlay::Element<'_, Message, Renderer>> {
        let position = Point::default();
        Some(
            TileOverlay::new(&mut self.state, &self.zoom_in).overlay(Point::new(0.0, 0.0)),
            //overlay::Element::new(position, Box::new(TileOverlay::new().overlay()))
            //    .overlay(Point::new(0.0, 0.0)),
        )
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
        translation: Vector,
        //handle: image::Handle,
        tile_handles: &[[Option<image::Handle>; 4]; 4],
    ) -> Self::Output;

    /// Decorates a the list of options of a [`MapTile`].
    ///
    /// This method can be used to draw a background for the [`Menu`].
    fn decorate(
        &mut self,
        bounds: Rectangle,
        //cursor_position: Point,
        //style: &<Self as Renderer>::Style,
        //primitive: Self::Output,
        tile_handles: &[[Option<image::Handle>; 4]; 4],
    ) -> Self::Output;
}

impl<'a, B, Message, Renderer> From<MapTile<'a, B, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    B: 'a + Fn(&mut button::State) -> Button<'_, Message, Renderer>,
    Message: 'a + Clone,
    Renderer: 'a + button::Renderer + iced_native::text::Renderer + self::Renderer,
{
    fn from(map_tile: MapTile<'a, B, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(map_tile)
    }
}

impl<B> Renderer for iced_graphics::Renderer<B>
where
    B: Backend + backend::Image,
{
    fn decorate(
        &mut self,
        bounds: Rectangle,
        //handle: image::Handle,
        //
        tile_handles: &[[Option<image::Handle>; 4]; 4],
    ) -> Self::Output {
        println!("hitting decorate");
        let mut primitives_vec: Vec<Primitive> = Vec::new();

        for (idx_x, x) in tile_handles.iter().enumerate() {
            for (idx_y, y) in x.iter().enumerate() {
                if let Some(tile) = &tile_handles[idx_x][idx_y] {
                    let top_left = Vector::new(idx_x as f32 * 256.0, idx_y as f32 * 256.0);

                    let new_clip = Primitive::Clip {
                        bounds,
                        content: Box::new(Primitive::Translate {
                            //translation,
                            translation: top_left,
                            content: Box::new(Primitive::Image {
                                handle: tile.clone(),
                                //bounds: Rectangle {
                                //    x: 0.0,
                                //    y: 0.0,
                                //    ..Rectangle::with_size(image_size)
                                //},
                                bounds: Rectangle {
                                    x: (idx_x * 256) as f32,
                                    y: (idx_y * 256) as f32,
                                    width: 256.0,
                                    height: 256.0,
                                },
                            }),
                        }),
                        offset: Vector::new((idx_x * 256) as u32, (idx_y * 256) as u32),
                    };
                    primitives_vec.push(new_clip);
                }
                //self.tile_handles[idx_x][idx_y] = Some(image::Handle::from_memory(tiles[idx_x][idx_y]));
            }
        }
        (
            {
                //
                Primitive::Group {
                    primitives: primitives_vec,
                }
            },
            { mouse::Interaction::Idle },
        )
    }

    fn draw(
        &mut self,
        bounds: Rectangle,
        translation: Vector,
        //handle: image::Handle,
        //
        tile_handles: &[[Option<image::Handle>; 4]; 4],
    ) -> Self::Output {
        let mut primitives_vec: Vec<Primitive> = Vec::new();

        for (idx_x, x) in tile_handles.iter().enumerate() {
            for (idx_y, y) in x.iter().enumerate() {
                if let Some(tile) = &tile_handles[idx_x][idx_y] {
                    let top_left = Vector::new(idx_x as f32 * 256.0, idx_y as f32 * 256.0);

                    let new_clip = Primitive::Clip {
                        bounds,
                        content: Box::new(Primitive::Translate {
                            //translation,
                            translation: top_left,
                            content: Box::new(Primitive::Image {
                                handle: tile.clone(),
                                //bounds: Rectangle {
                                //    x: 0.0,
                                //    y: 0.0,
                                //    ..Rectangle::with_size(image_size)
                                //},
                                bounds: Rectangle {
                                    x: (idx_x * 256) as f32,
                                    y: (idx_y * 256) as f32,
                                    width: 256.0,
                                    height: 256.0,
                                },
                            }),
                        }),
                        offset: Vector::new((idx_x * 256) as u32, (idx_y * 256) as u32),
                    };
                    primitives_vec.push(new_clip);
                }
                //self.tile_handles[idx_x][idx_y] = Some(image::Handle::from_memory(tiles[idx_x][idx_y]));
            }
        }
        (
            {
                //
                Primitive::Group {
                    primitives: primitives_vec,
                }
            },
            { mouse::Interaction::Idle },
        )
    }
}
