//use bytes::{BufMut, Bytes, BytesMut};
// For now, to implement a custom native widget you will need to add
// `iced_native` and `iced_wgpu` to your dependencies.
//
// Then, you simply need to define your widget type and implement the
// `iced_native::Widget` trait with the `iced_wgpu::Renderer`.
//
// Of course, you can choose to make the implementation renderer-agnostic,
// if you wish to, by creating your own `Renderer` trait, which could be
// implemented by `iced_wgpu` and other renderers.
use crate::widgets::map_tile_overlay::TileOverlay;
use iced::image;
use iced_graphics::backend::{self, Backend};
use iced_graphics::Primitive;
use iced_native::event;
use iced_native::mouse::click;
use iced_native::{
    button, layout, mouse, overlay, touch, Button, Clipboard, Element, Event, Hasher, Layout,
    Length, Point, Rectangle, Size, Vector, Widget,
};

use log;

pub const TILE_DIMENSION: usize = 5;

pub struct MapTile<'a, B> {
    state: &'a mut State,
    zoom_in_state: &'a mut button::State,
    zoom_out_state: &'a mut button::State,
    zoom_in: B,
    zoom_out: B,
    tile_handles: [[Option<image::Handle>; TILE_DIMENSION]; TILE_DIMENSION],
    width: Length,
    height: Length,
}

impl<'a, B, Message, Renderer> Widget<Message, Renderer> for MapTile<'a, B>
where
    //B: fn(&mut button::State) -> Button<'_, Message>,
    B: Fn(&mut button::State) -> Button<'_, Message, Renderer>,
    Message: 'a + Clone,
    Renderer: 'a
        + self::Renderer
        + iced_native::image::Renderer
        + iced_native::Renderer
        + iced_native::text::Renderer
        + iced_native::button::Renderer,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        //let (width, height) = renderer.dimensions(&self.handle);
        let (width, height) = (256.0 * 3.0, 256.0 * 3.0);
        layout::Node::new(limits.resolve(Size::new(width, height)))
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

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let is_clicked = bounds.contains(cursor_position);
                self.state.is_focused = is_clicked;
                let click = mouse::Click::new(cursor_position, self.state.last_click);

                match click.kind() {
                    click::Kind::Single => {
                        self.state.is_dragging = true;
                    }
                    _ => {}
                }

                self.state.last_click = Some(click);
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                self.state.is_dragging = false;
            }
            Event::Mouse(mouse::Event::CursorMoved { position })
            | Event::Touch(touch::Event::FingerMoved { position, .. }) => {
                if self.state.is_dragging {
                    self.state.velocity = (
                        position.x - self.state.last_position.0,
                        position.y - self.state.last_position.1,
                    );

                    log::trace!(
                        "vel is {}, {}",
                        self.state.velocity.0, self.state.velocity.1
                    );
                }
                else{
                    self.state.velocity =(0.0,0.0);
                }
                self.state.last_position = (position.x, position.y);
                self.state.load_pixel.0 += -self.state.velocity.0;
                self.state.load_pixel.1 += -self.state.velocity.1;
            }

            _ => {}
        }

        event::Status::Captured
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
        self::Renderer::draw(
            renderer,
            bounds,
            translation,
            &self.tile_handles,
            self.state.load_pixel,
        )
        //renderer.draw(self.handle.clone(), layout)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;
        //TODO: proper hashing of layout
        //self.state.load_pixel.hash(state);
        self.tile_handles.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }

    /// Returns the overlay of the [`Widget`], if there is any.
    fn overlay(&mut self, layout: Layout<'_>) -> Option<overlay::Element<'_, Message, Renderer>> {
        let zoom_in = (self.zoom_in)(&mut self.zoom_in_state);
        let zoom_out = (self.zoom_out)(&mut self.zoom_out_state);
        let edge_x = layout.bounds().x + layout.bounds().width - 125.0;
        let edge_y = layout.bounds().y + layout.bounds().height - 125.0;
        Some(
            TileOverlay::new(zoom_in, zoom_out).overlay(Point::new(
                f32::min(edge_x, 256.0 * 3.0 - 125.0),
                f32::min(edge_y, 256.0 * 3.0 - 125.0),
            )),
            //overlay::Element::new(position, Box::new(TileOverlay::new().overlay()))
            //    .overlay(Point::new(0.0, 0.0)),
        )
    }
}

/// The state of a [`MapTile`].
#[derive(Debug, Default, Clone, Copy)]
pub struct State {
    is_focused: bool,
    is_dragging: bool,
    last_position: (f32, f32),
    pub velocity: (f32, f32),
    pub load_pixel: (f32, f32),
    last_click: Option<mouse::Click>,
}

impl State {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    pub fn new(
        is_dragging: bool,
        last_position: (f32, f32),
        last_click: Option<mouse::Click>,
        load_pixel: (f32, f32),
    ) -> Self {
        Self {
            is_focused: false,
            is_dragging,
            last_position,
            velocity: (0.0, 0.0),
            load_pixel,
            last_click,
        }
    }

    /// Returns whether the [`TextInput`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Focuses the [`TextInput`].
    pub fn focus(&mut self) {
        self.is_focused = true;
    }

    /// Unfocuses the [`TextInput`].
    pub fn unfocus(&mut self) {
        self.is_focused = false;
    }
}

impl<'a, B, Message, Renderer> MapTile<'a, B>
where
    Message: Clone,
    Renderer: self::Renderer + iced_native::button::Renderer,
    B: Fn(&mut button::State) -> Button<'_, Message, Renderer>,
{
    pub fn new(
        state: &'a mut State,
        tiles: [[Vec<u8>; TILE_DIMENSION]; TILE_DIMENSION],
        zoom_in_state: &'a mut button::State,
        zoom_out_state: &'a mut button::State,
        zoom_in: B,
        zoom_out: B,
    ) -> Self {
        let mut tile_handles: [[Option<image::Handle>; TILE_DIMENSION]; TILE_DIMENSION] =
            Default::default();

        for (idx_x, x) in tiles.iter().enumerate() {
            for (idx_y, y) in x.iter().enumerate() {
                tile_handles[idx_x][idx_y] =
                    Some(image::Handle::from_memory(tiles[idx_x][idx_y].clone()));
            }
        }

        //let tile_handles = image::Handle::from_memory(bytes.to_vec());
        Self {
            state,
            zoom_in_state,
            zoom_out_state,
            zoom_in,
            zoom_out,
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
/// The renderer of an [`Viewer`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Viewer`] in your user interface.
///
/// [renderer]: crate::renderer
pub trait Renderer:
    iced_native::Renderer
    + iced_native::image::Renderer
    + iced_native::button::Renderer
    + iced_native::text::Renderer
    + Sized
{
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
        tile_handles: &[[Option<image::Handle>; TILE_DIMENSION]; TILE_DIMENSION],
        load_point: (f32, f32),
    ) -> Self::Output;

    fn overlay_draw<Message: Clone>(
        &mut self,
        defaults: &Self::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        zoom_in: &iced_native::Button<'_, Message, Self>,
        zoom_out: &iced_native::Button<'_, Message, Self>,
    ) -> Self::Output;

    //fn draw<Message>(
    //    &mut self,
    //    env: DrawEnvironment<'_, Self::Defaults, Self::Style, Focus>,
    //    color: &Color,
    //    sat_value_canvas_cache: &canvas::Cache,
    //    hue_canvas_cache: &canvas::Cache,
    //    //text_input: &Element<'_, Message, Self>,
    //    cancel_button: &Element<'_, Message, Self>,
    //    submit_button: &Element<'_, Message, Self>,
    //) -> Self::Output;
}

impl<B> Renderer for iced_graphics::Renderer<B>
where
    B: Backend + backend::Image + backend::Text + backend::Backend,
{
    fn draw(
        &mut self,
        bounds: Rectangle,
        translation: Vector,
        //handle: image::Handle,
        //
        tile_handles: &[[Option<image::Handle>; TILE_DIMENSION]; TILE_DIMENSION],
        load_point: (f32, f32),
    ) -> Self::Output {
        let mut primitives_vec: Vec<Primitive> = Vec::new();
        log::info!("load point {}, {}",load_point.0, load_point.1);
        let load_point = (load_point.0/256.0, load_point.1/256.0);
        let load_top_left = ((load_point.0 - 1.5) * 256.0, (load_point.1 - 1.5) * 256.0);
        let load_bottom_right = ((load_point.0 + 1.5) * 256.0, (load_point.1 + 1.5) * 256.0);
        for (idx_x, x) in tile_handles.iter().enumerate() {
            for (idx_y, y) in x.iter().enumerate() {
                let handle_top_left = ((idx_x as f32 - 2.5) * 256.0, (idx_y as f32 - 2.5) * 256.0);
                let handle_bottom_right =
                    ((idx_x as f32 - 1.5) * 256.0, (idx_y as f32 - 1.5) * 256.0);
                if let Some(tile) = &tile_handles[idx_x][idx_y] {
                    //let top_left = Vector::new(idx_x as f32 * 0.0, idx_y as f32 * 0.0);
                    let mut x = load_top_left.0 - handle_top_left.0;
                    let mut y = load_top_left.1 - handle_top_left.1;

                    let mut width = handle_top_left.0 - load_bottom_right.0;
                    let mut height = handle_top_left.1 - load_bottom_right.1;
                    log::debug!(
                        "comparing {},{} - {},{} to {},{} - {},{}",
                        load_top_left.0,
                        load_top_left.1,
                        load_bottom_right.0,
                        load_bottom_right.1,
                        handle_top_left.0,
                        handle_top_left.1,
                        handle_bottom_right.0,
                        handle_bottom_right.1
                    );

                    log::debug!("raw vals {}, {}, {}, {}", x, y, width, height);
                    if handle_bottom_right.0 <= load_top_left.0
                        || handle_bottom_right.1 <= load_top_left.1
                    {
                        log::debug!("skipping tile {}, {}", idx_x, idx_y);
                        continue;
                    }
                    if handle_top_left.0 >= load_bottom_right.0
                        || handle_top_left.1 >= load_bottom_right.1
                    {
                        log::debug!("skipping tile {}, {}", idx_x, idx_y);
                        continue;
                    }

                    //if the top left handle is before the top left load, for a dimension, then
                    //that dimension is less than 256 and x/y compensate
                    //if the bottom right handle is greater than the bottom right load, then
                    //that dimension is less than 256 and x/y remain 0
                    //otherwise, the entire tile is swallowed

                    if handle_top_left.0 < load_top_left.0 {
                        width = handle_bottom_right.0 - load_top_left.0;
                        x = 256. - width;
                    } else if handle_bottom_right.0 > load_bottom_right.0 {
                        width = load_bottom_right.0 - handle_top_left.0;
                        x = 0.0;
                    } else {
                        width = 256.0;
                        x = 0.0;
                    }

                    if handle_top_left.1 < load_top_left.1 {
                        height = handle_bottom_right.1 - load_top_left.1;
                        y = 256. - height;
                    } else if handle_bottom_right.1 > load_bottom_right.1 {
                        height = load_bottom_right.1 - handle_top_left.1;
                        y = 0.0;
                    } else {
                        height = 256.0;
                        y = 0.0;
                    }

                    let pixel_x = handle_top_left.0 - load_top_left.0 as f32 + x;
                    let pixel_y = handle_top_left.1 - load_top_left.1 as f32 + y;

                    log::info!(
                        "putting tile {}, {} at {}, {}: {}x{}",
                        idx_x,
                        idx_y,
                        pixel_x,
                        pixel_y,
                        width,
                        height
                    );
                    //x = 0.0;
                    //y =0.0;
                    //width = 256.0;
                    //height=256.0;
                    let new_clip = Primitive::Image {
                        handle: tile.clone(),
                        //bounds: Rectangle {
                        //    x: 0.0,
                        //    y: 0.0,
                        //    ..Rectangle::with_size(image_size)
                        //},
                        bounds: Rectangle {
                            x: pixel_x,
                            y: pixel_y,
                            width,
                            height,
                        },
                    };
                    primitives_vec.push(new_clip);
                }
                //self.tile_handles[idx_x][idx_y] = Some(image::Handle::from_memory(tiles[idx_x][idx_y]));
            }
        }

        //changing the quad here changes nothing
        //let new_quad = Primitive::Quad {
        //    bounds: Rectangle {
        //        x: (0 * 256) as f32,
        //        y: (0 * 256) as f32,
        //        width: 256.0,
        //        height: 256.0,
        //    },
        //    background: Background::Color(Color::BLACK),
        //    border_radius: 40.0,
        //    border_width: 0.0,
        //    border_color: Color::TRANSPARENT,
        //};
        (
            {
                //
                Primitive::Group {
                    primitives: primitives_vec,
                }
            },
            { mouse::Interaction::Grab },
            //{ mouse::Interaction::Idle },
        )
    }

    fn overlay_draw<Message: Clone>(
        &mut self,
        defaults: &Self::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        zoom_in: &iced_native::Button<'_, Message, Self>,
        zoom_out: &iced_native::Button<'_, Message, Self>,
    ) -> Self::Output {
        let bounds = layout.bounds();
        let mouse_interaction = mouse::Interaction::default();
        let mut children = layout.children();
        let zoom_in_layout = children
            .next()
            .expect("Native: layout should have zoom in button for MapTile");
        let zoom_out_layout = children
            .next()
            .expect("Native: layout should have zoom out button for MapTile");

        let (zoom_in_button, zoom_in_interaction) =
            zoom_in.draw(self, defaults, zoom_in_layout, cursor_position, &bounds);

        let (zoom_out_button, zoom_out_interaction) =
            zoom_out.draw(self, defaults, zoom_out_layout, cursor_position, &bounds);
        (
            Primitive::Group {
                primitives: vec![zoom_in_button, zoom_out_button],
            },
            mouse_interaction
                .max(zoom_in_interaction)
                .max(zoom_out_interaction),
        )
    }
}

//impl<'a, Message, B> Into<Element<'a, Message, Renderer<B>>> for Circle
//impl<'a, B, Message, Renderer> Into<Element<'a, Message, Renderer>> for MapTile<'a, B, Message, Renderer>
impl<'a, B, Message, Renderer> Into<Element<'a, Message, Renderer>> for MapTile<'a, B>
where
    B: 'a + Fn(&mut button::State) -> Button<'_, Message, Renderer>,
    Message: 'a + Clone,
    Renderer: 'a
        + self::Renderer
        + iced_native::image::Renderer
        + iced_native::text::Renderer
        + iced_native::button::Renderer,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
