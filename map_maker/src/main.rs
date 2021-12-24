// Forbid warnings in release builds:
//#![cfg_attr(not(debug_assertions), deny(warnings))]
#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
mod tile_manager;
mod widgets;
use futures::future::join_all;
use log;
use std::sync::Arc;
use thiserror::Error;
use widgets::map_tile;
use Result;

use env_logger::{Builder, Target};
use tile_manager::tile_manager::Tile;
use tile_manager::tile_manager::TileManager;
use tile_manager::tile_manager::TileState;

pub const LOAD_TILE_DIMENSION: usize = 5;
use crate::widgets::map_tile::TILE_DIMENSION;

use iced::{
    button, executor, Application, Button, Command, Container, Element, Length, Settings, Text,
};

use slippy_map_tiles;
//fn tokio_runtime_thread(tx: Sender<Bytes>) {
//    let mut rt = Runtime::new().unwrap();
//    let handle = rt.spawn(async move {
//        let resp = reqwest::get("https://stamen-tiles.a.ssl.fastly.net/terrain/2/1/3.png")
//            .await?
//            .bytes()
//            .await?;
//        tx.send(resp).await;
//        Ok::<(), reqwest::Error>(()) // <- note the explicit type annotation here
//    });
//    rt.block_on(handle);
//}
pub fn main() -> iced::Result {
    //let (tx, mut rx) = mpsc::channel(100);
    //let tokio_thread_handle = thread::spawn(|| tokio_runtime_thread(tx));
    //

    //
    //store a latlon in the main structure
    //drag events shift pixels and every pixel update, we update the latlon as well
    //
    //
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.filter(Some("map_maker"), log::LevelFilter::Info);
    builder.init();
    let result = MapMaker::run(Settings::default());
    //tokio_thread_handle.join().unwrap();
    result
}

struct MapMaker {
    //
    //should store an arry of vectors
    //each vector is an image tile
    //the widget handles loading the image tiles
    //
    tiles: [[Tile; LOAD_TILE_DIMENSION]; LOAD_TILE_DIMENSION],
    zoom_in_state: button::State,
    zoom_out_state: button::State,
    cur_coords: (f32, f32),
    load_pixel: (f32, f32),
    zoom_level: u8,
    client: std::sync::Arc<reqwest::Client>,
    tile_state: map_tile::State,
    tile_manager: TileManager,
}

//slippy_map_tiles::lat_lon_to_tile
#[derive(Clone, Debug)]
pub enum MyMessage {
    LoadedImage(Vec<Tile>),
    ZoomIn,
    ZoomOut,
    ImageLoadFailed,
    CenterPosition,
    VelocityEvent,
}

#[derive(Debug, Error)]
enum MyError {
    #[error("api error")]
    APIError,
    #[error("https error")]
    HttpsError(#[from] reqwest::Error),
}

impl MapMaker {
    //tile_handles: [[Option<image::Handle>; TILE_DIMENSION]; TILE_DIMENSION],
    //
    fn get_tile_imgs(&self) -> [[Vec<u8>; LOAD_TILE_DIMENSION]; LOAD_TILE_DIMENSION] {
        let mut imgs: [[Vec<u8>; LOAD_TILE_DIMENSION]; LOAD_TILE_DIMENSION] = Default::default();
        for x in 0..LOAD_TILE_DIMENSION {
            for y in 0..LOAD_TILE_DIMENSION {
                imgs[x][y] = self.tiles[x][y].image.clone();
            }
        }
        imgs
    }

    fn populate_tiles(&mut self) {
        let center_x = self.load_pixel.0;
        let center_y = self.load_pixel.1;
        let center_tile: (isize, isize) = (center_x as isize / 256, center_y as isize / 256);
        let zoom_level = self.zoom_level;

        for x in 0..LOAD_TILE_DIMENSION {
            for y in 0..LOAD_TILE_DIMENSION {
                let tile_x = center_tile.0 + (x as isize - 2);
                let tile_y = center_tile.1 + (y as isize - 2);
                if tile_x > 0 && tile_y > 0 {
                    let target_url = (tile_x as u32, tile_y as u32, zoom_level as u32);
                    let target_tile = self.tile_manager.get_tile(&(&target_url));

                    if let TileState::NotLoaded = target_tile.state {
                        self.tile_manager.queue_tile_load(target_url);
                        log::info!(
                            "me no have tile, queueing ({},{},{})",
                            target_url.0,
                            target_url.1,
                            target_url.2
                        );
                    }
                    self.tiles[x][y] = target_tile;
                }
            }
        }
    }

    async fn velocity_wait() {
        tokio::time::sleep(std::time::Duration::new(0, 1e7 as u32)).await;
        //(Duration::from_secs(3)).await;
    }

    fn process_load(resp: Option<Vec<Tile>>) -> MyMessage {
        match resp {
            Some(tiles) => MyMessage::LoadedImage(tiles),
            None => MyMessage::ImageLoadFailed,
        }
    }

    fn print_tiles(&self) {
        for x in 0..LOAD_TILE_DIMENSION {
            for y in 0..LOAD_TILE_DIMENSION {
                print!(
                    "({},{},{}) ",
                    self.tiles[x][y].target_url.0,
                    self.tiles[x][y].target_url.1,
                    self.tiles[x][y].target_url.2,
                );
            }
            println!("");
        }
    }
}
impl Application for MapMaker {
    type Executor = executor::Default;
    type Message = MyMessage;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<MyMessage>) {
        // strange syntax
        //let tiles: [[Vec<u8>; 4]; 4] = [[Vec::new(); 4]; 4];
        let zoom_level: u8 = 4;

        //println!(
        //    "my position tile is {}",
        //    slippy_map_tiles::lat_lon_to_tile(42.473882, -83.473203, 3)
        //);
        let client = std::sync::Arc::new(reqwest::Client::new());
        (
            MapMaker {
                //TODO: add a new function that handles initializing the array
                tiles: Default::default(),
                zoom_in_state: button::State::new(),
                zoom_out_state: button::State::new(),
                cur_coords: (42.473882, -83.473203),
                zoom_level,
                load_pixel: (256.0 * 4.0, 256.0 * 5.0),
                client: client.clone(),
                tile_state: map_tile::State::default(),
                tile_manager: TileManager::new(),
            },
            //TODO: get the tile manager to spit out the async function to run
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("MapMaker")
    }

    fn update(&mut self, message: MyMessage) -> Command<MyMessage> {
        match message {
            MyMessage::LoadedImage(tiles) => {
                //TODO - when we load new images stuff them into the tile manager and check if they
                //belong in the current view
                if tiles.len() > 0 {
                    self.tile_manager.ingest_loaded_tiles(tiles);
                    self.populate_tiles();
                    return Command::perform(
                        self.tile_manager.generate_async_load(),
                        MapMaker::process_load,
                    );
                }
                return Command::none();
            }
            MyMessage::ZoomIn => {
                log::info!("me zoom in");

                self.zoom_level += 1;
                self.load_pixel.0 = self.load_pixel.0 * 2.;
                self.load_pixel.1 = self.load_pixel.1 * 2.;
                self.populate_tiles();
                return Command::perform(
                    self.tile_manager.generate_async_load(),
                    MapMaker::process_load,
                );
            }
            MyMessage::ZoomOut => {
                println!("me zoom out");
                self.zoom_level -= 1;
                self.load_pixel.0 = self.load_pixel.0 * 0.5;
                self.load_pixel.1 = self.load_pixel.1 * 0.5;
                self.populate_tiles();
                return Command::perform(
                    self.tile_manager.generate_async_load(),
                    MapMaker::process_load,
                );
            }

            MyMessage::ImageLoadFailed => {
                log::error!("image load failed");
            }

            MyMessage::VelocityEvent => {
                self.tile_state.vel_requested = false;
                if self.tile_state.velocity.0 == 0.0 && self.tile_state.velocity.1 == 0.0 {
                    return Command::none();
                }

                if self.tile_state.is_dragging == false {
                    log::info!(
                        "velocity event ({},{})",
                        self.tile_state.velocity.0,
                        self.tile_state.velocity.1
                    );
                    if self.tile_state.velocity.0 != 0.0 {
                        let decrementer = {
                            if self.tile_state.velocity.0.abs() < 0.1 {
                                -self.tile_state.velocity.0
                            } else {
                                if self.tile_state.velocity.0 > 0.0 {
                                    -0.1
                                } else {
                                    0.1
                                }
                            }
                        };

                        self.tile_state.velocity.0 += decrementer;
                    }

                    if self.tile_state.velocity.1 != 0.0 {
                        let decrementer = {
                            if self.tile_state.velocity.1.abs() < 0.1 {
                                -self.tile_state.velocity.1
                            } else {
                                if self.tile_state.velocity.1 > 0.0 {
                                    -0.1
                                } else {
                                    0.1
                                }
                            }
                        };

                        self.tile_state.velocity.1 += decrementer;
                    }
                    self.tile_state.load_pixel.0 += -self.tile_state.velocity.0;
                    self.tile_state.load_pixel.1 += -self.tile_state.velocity.1;

                    if self.tile_state.center_requested == false
                        && self.tile_state.load_pixel.0.abs() > 256.0
                        || self.tile_state.load_pixel.1.abs() > 256.0
                    {
                        log::trace!("requesting centering");
                        self.tile_state.center_requested = true;
                        self.tile_state.vel_requested = true;

                        Command::batch(vec![
                            Command::perform(MapMaker::velocity_wait(), |_| {
                                MyMessage::VelocityEvent
                            }),
                            Command::perform(async {}, |_| MyMessage::CenterPosition),
                        ]);

                        //return Command::perform(
                        //    MapMaker::velocity_wait(),
                        //    |_| {

                        //    Command::batch(vec![
                        //        MyMessage::CenterPosition,
                        //        MyMessage::VelocityEvent,
                        //    ])},
                        //);
                    }
                }
                self.tile_state.vel_requested = true;
                return Command::perform(MapMaker::velocity_wait(), |_| MyMessage::VelocityEvent);
            }

            MyMessage::CenterPosition => {
                //change the load pixel back to something centered
                //and start loading tiles to adjust for the change
                //TODO: start the load
                let mut x_delta = 0.0;
                let mut y_delta = 0.0;
                self.tile_state.load_pixel;
                while self.tile_state.load_pixel.0.abs() > 256.0 {
                    if self.tile_state.load_pixel.0 < -256.0 {
                        self.tile_state.load_pixel.0 += 256.0;
                        x_delta = -256.0;
                    } else if self.tile_state.load_pixel.0 > 256.0 {
                        self.tile_state.load_pixel.0 -= 256.0;
                        x_delta = 256.0;
                    }
                }

                while self.tile_state.load_pixel.1.abs() > 256.0 {
                    if self.tile_state.load_pixel.1 < -256.0 {
                        self.tile_state.load_pixel.1 += 256.0;
                        y_delta = -256.0;
                    } else if self.tile_state.load_pixel.1 > 256.0 {
                        self.tile_state.load_pixel.1 -= 256.0;
                        y_delta = 256.0;
                    }
                }

                //MapMaker::shift_tiles(&mut self.tiles, row_shift, col_shift);
                //let request_tiles = self.get_request_tiles();
                self.tile_state.center_requested = false;
                self.load_pixel.0 += x_delta;
                self.load_pixel.1 += y_delta;
                self.populate_tiles();
                self.print_tiles();
                return Command::perform(
                    self.tile_manager.generate_async_load(),
                    MapMaker::process_load,
                );
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, MyMessage> {
        fn zoom_in_spawner(state: &mut button::State) -> Button<'_, MyMessage> {
            Button::new(state, Text::new("zoom in")).on_press(MyMessage::ZoomIn)
        }
        fn zoom_out_spawner(state: &mut button::State) -> Button<'_, MyMessage> {
            Button::new(state, Text::new("zoom out")).on_press(MyMessage::ZoomOut)
        }
        type ButtonSpawner = fn(&mut button::State) -> Button<'_, MyMessage>;
        //let content = map_tile::MapTile::new(self.tiles.clone(), &mut self.button_state, zoom_spawner);

        //let content = map_tile::MapTile::new(self.tiles.clone(), &mut self.button_state, |state|-> Button<'_, Message>{
        //    Button::new(state, Text::new("Press Me!")).on_press(Message::ButtonPressed)
        //});
        //cannot call this function in the container declaration because of borrowing rules
        let imgs = self.get_tile_imgs();
        Container::new(map_tile::MapTile::new(
            &mut self.tile_state,
            imgs,
            &mut self.zoom_in_state,
            &mut self.zoom_out_state,
            //https://stackoverflow.com/questions/27895946/expected-fn-item-found-a-different-fn-item-when-working-with-function-pointer
            zoom_in_spawner as ButtonSpawner,
            zoom_out_spawner as ButtonSpawner,
            MyMessage::CenterPosition,
            MyMessage::VelocityEvent,
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}
