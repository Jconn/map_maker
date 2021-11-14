// Forbid warnings in release builds:
//#![cfg_attr(not(debug_assertions), deny(warnings))]
#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
mod widgets;
use futures::future::join_all;
use log;
use std::sync::Arc;
use thiserror::Error;
use widgets::map_tile;
use Result;

use env_logger::{Builder, Target};

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
    tiles: [[Vec<u8>; LOAD_TILE_DIMENSION]; LOAD_TILE_DIMENSION],
    zoom_in_state: button::State,
    zoom_out_state: button::State,
    cur_coords: (f32, f32),
    load_pixel: (f32, f32),
    zoom_level: u8,
    client: std::sync::Arc<reqwest::Client>,
    tile_state: map_tile::State,
}

#[derive(Clone, Debug)]
struct Tile {
    target_url: (u32, u32, u32),
    dest_tile: (u32, u32),
    image: Vec<u8>,
}
//slippy_map_tiles::lat_lon_to_tile
impl Tile {
    fn new(target_url: (u32, u32, u32), dest_tile: (u32, u32)) -> Self {
        Self {
            target_url,
            dest_tile,
            image: Vec::new(),
        }
    }
}
#[derive(Clone, Debug)]
enum Message {
    LoadedImage(Vec<Tile>),
    ZoomIn,
    ZoomOut,
    ImageLoadFailed,
    CenterPosition,
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

    async fn get_tile(
        mut request_tile: Tile,
        client: Arc<reqwest::Client>,
    ) -> Result<Tile, MyError> {
        let (x, y, z) = request_tile.target_url;
        let resp = client
            .get(format!(
                "https://stamen-tiles.a.ssl.fastly.net/terrain/{}/{}/{}.png",
                z, x, y
            ))
            .send()
            .await?
            .bytes()
            .await?
            .to_vec();
        request_tile.image = resp;
        Ok(request_tile)
    }

    async fn load(
        client: Arc<reqwest::Client>,
        load_tiles: Vec<Tile>,
    ) -> Result<Vec<Tile>, MyError> {
        let mut return_tiles = load_tiles.clone();

        //type FutureType = Box<dyn Future<Output = Result<Tile, MyError> > + Unpin  >;
        //type FutureType = fn(mut Tile,  Arc<reqwest::Client>) ->Result<Tile, MyError>;
        //let mut tile_futures: Vec<FutureType> = Vec::new();
        //for tile in &mut return_tiles {
        //    tile_futures.push(Box::new(Box::pin(MapMaker::get_tile(tile.clone(), client.clone()))));
        //}

        let tile_futures = return_tiles
            .iter_mut()
            .map(|tile| MapMaker::get_tile(tile.clone(), client.clone()));

        let tile_results = join_all(tile_futures).await;

        for tile_result in tile_results {
            if let Ok(tile) = tile_result {
                return_tiles.push(tile);
            }
        }
        Ok(return_tiles)
    }

    fn process_load(resp: Result<Vec<Tile>, MyError>) -> Message {
        match resp {
            Ok(tiles) => Message::LoadedImage(tiles),
            Err(err) => Message::ImageLoadFailed,
        }
    }

    fn generate_tiles(&self) -> Vec<Tile> {
        let target_tile = slippy_map_tiles::lat_lon_to_tile(
            self.cur_coords.0,
            self.cur_coords.1,
            self.zoom_level,
        );
        let mut tiles: Vec<Tile> = Vec::new();

        for x in 0..LOAD_TILE_DIMENSION {
            for y in 0..LOAD_TILE_DIMENSION {
                tiles.push(Tile::new(
                    (
                        target_tile.0 + x as u32 - 1,
                        target_tile.1 + y as u32 - 1,
                        self.zoom_level as u32,
                    ),
                    (x as u32, y as u32),
                ));
            }
        }
        tiles
    }

    fn shift_tiles(
        tiles: &mut [[Vec<u8>; LOAD_TILE_DIMENSION]; LOAD_TILE_DIMENSION],
        row: i32,
        col: i32,
    ) {
        fn rotate_col(
            tiles: &mut [[Vec<u8>; LOAD_TILE_DIMENSION]; LOAD_TILE_DIMENSION],
            col_rot: i32,
        ) {
            for t_row in tiles {
                if col_rot > 0 {
                    t_row.rotate_right(col_rot as usize);
                    t_row[0..col_rot as usize].fill(Default::default());
                } else if col_rot < 0 {
                    t_row.rotate_left(-col_rot as usize);
                    t_row[(LOAD_TILE_DIMENSION as i32 + col_rot) as usize
                        ..LOAD_TILE_DIMENSION as usize]
                        .fill(Default::default());
                }
            }
        }
        if row > 0 {
            tiles.rotate_right(row as usize);
            tiles[0..row as usize].fill(Default::default());
        } else if row < 0 {
            tiles.rotate_left(-row as usize);
            tiles[(LOAD_TILE_DIMENSION as i32 + row) as usize..LOAD_TILE_DIMENSION]
                .fill(Default::default());
        }

        rotate_col(tiles, col);
    }
}
impl Application for MapMaker {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        // strange syntax
        //let tiles: [[Vec<u8>; 4]; 4] = [[Vec::new(); 4]; 4];
        let tiles: [[Vec<u8>; LOAD_TILE_DIMENSION]; LOAD_TILE_DIMENSION] = Default::default();
        let zoom_level: u8 = 4;
        let mut request_tiles: Vec<Tile> = Vec::new();
        for x in 0..LOAD_TILE_DIMENSION {
            for y in 0..LOAD_TILE_DIMENSION {
                request_tiles.push(Tile::new(
                    (x as u32, y as u32, zoom_level as u32),
                    (x as u32, y as u32),
                ));
            }
        }

        //println!(
        //    "my position tile is {}",
        //    slippy_map_tiles::lat_lon_to_tile(42.473882, -83.473203, 3)
        //);
        let client = std::sync::Arc::new(reqwest::Client::new());
        (
            MapMaker {
                //TODO: add a new function that handles initializing the array
                tiles: tiles.clone(),
                zoom_in_state: button::State::new(),
                zoom_out_state: button::State::new(),
                cur_coords: (42.473882, -83.473203),
                zoom_level,
                load_pixel: (0.0, 0.0),
                client: client.clone(),
                tile_state: map_tile::State::default(),
            },
            Command::perform(
                MapMaker::load(client, request_tiles),
                MapMaker::process_load,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("MapMaker")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadedImage(tiles) => {
                for tile in tiles {
                    let (x, y) = tile.dest_tile;
                    self.tiles[x as usize][y as usize] = tile.image;
                }
            }
            Message::ZoomIn => {
                log::info!("me zoom in");

                self.zoom_level += 1;
                let request_tiles = self.generate_tiles();
                return Command::perform(
                    MapMaker::load(self.client.clone(), request_tiles),
                    MapMaker::process_load,
                );
            }
            Message::ZoomOut => {
                println!("me zoom out");
                self.zoom_level -= 1;
                let request_tiles = self.generate_tiles();
                return Command::perform(
                    MapMaker::load(self.client.clone(), request_tiles),
                    MapMaker::process_load,
                );
            }

            Message::ImageLoadFailed => {
                log::error!("image load failed");
            }

            Message::CenterPosition => {
                //change the load pixel back to something centered
                //and start loading tiles to adjust for the change
                //TODO: start the load
                let mut col_shift = 0;
                let mut row_shift = 0;
                self.tile_state.load_pixel;
                if self.tile_state.load_pixel.0 < -256.0 {
                    self.tile_state.load_pixel.0 += 256.0;
                    row_shift = 1;
                } else if self.tile_state.load_pixel.0 > 256.0 {
                    self.tile_state.load_pixel.0 -= 256.0;
                    row_shift = -1;
                }

                if self.tile_state.load_pixel.1 < -256.0 {
                    self.tile_state.load_pixel.1 += 256.0;
                    col_shift = 1;
                } else if self.tile_state.load_pixel.1 > 256.0 {
                    self.tile_state.load_pixel.1 -= 256.0;
                    col_shift = -1;
                }

                MapMaker::shift_tiles(&mut self.tiles, row_shift, col_shift);
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Message> {
        fn zoom_in_spawner(state: &mut button::State) -> Button<'_, Message> {
            Button::new(state, Text::new("zoom in")).on_press(Message::ZoomIn)
        }
        fn zoom_out_spawner(state: &mut button::State) -> Button<'_, Message> {
            Button::new(state, Text::new("zoom out")).on_press(Message::ZoomOut)
        }
        type ButtonSpawner = fn(&mut button::State) -> Button<'_, Message>;
        //let content = map_tile::MapTile::new(self.tiles.clone(), &mut self.button_state, zoom_spawner);

        //let content = map_tile::MapTile::new(self.tiles.clone(), &mut self.button_state, |state|-> Button<'_, Message>{
        //    Button::new(state, Text::new("Press Me!")).on_press(Message::ButtonPressed)
        //});

        Container::new(map_tile::MapTile::new(
            &mut self.tile_state,
            self.tiles.clone(),
            &mut self.zoom_in_state,
            &mut self.zoom_out_state,
            //https://stackoverflow.com/questions/27895946/expected-fn-item-found-a-different-fn-item-when-working-with-function-pointer
            zoom_in_spawner as ButtonSpawner,
            zoom_out_spawner as ButtonSpawner,
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}
