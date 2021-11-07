// Forbid warnings in release builds:
//#![cfg_attr(not(debug_assertions), deny(warnings))]
#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
mod widgets;
use bytes::Bytes;
use std::thread;
use thiserror::Error;
use widgets::map_tile;
use Result;

use iced::{
    button, executor, slider, Alignment, Application, Button, Column, Command, Container, Element,
    Length, Sandbox, Settings, Slider, Text,
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
    tiles: [[Vec<u8>; 4]; 4],
    zoom_in_state: button::State,
    zoom_out_state: button::State,
    cur_coords: (f32, f32),
    zoom_level: u8,
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
}

#[derive(Debug, Error)]
enum MyError {
    #[error("api error")]
    APIError,
    #[error("https error")]
    HttpsError(#[from] reqwest::Error),
}

impl MapMaker {
    async fn load(load_tiles: Vec<Tile>) -> Result<Vec<Tile>, MyError> {
        let mut return_tiles = load_tiles.clone();
        println!("me trying load");
        for mut tile in &mut return_tiles {
            let (x, y, z) = tile.target_url;
            println!("loading {}, {}, {}", x, y, z);
            let resp = reqwest::get(format!(
                "https://stamen-tiles.a.ssl.fastly.net/terrain/{}/{}/{}.png",
                z, x, y
            ))
            .await?
            .bytes()
            .await?
            .to_vec();
            tile.image = resp;
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

        for x in 0..4 {
            for y in 0..4 {
                tiles.push(Tile::new(
                    (
                        target_tile.0 + x - 2,
                        target_tile.1 + y - 2,
                        self.zoom_level as u32,
                    ),
                    (x, y),
                ));
            }
        }
        tiles
    }
}
impl Application for MapMaker {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        // strange syntax
        //let tiles: [[Vec<u8>; 4]; 4] = [[Vec::new(); 4]; 4];
        let tiles: [[Vec<u8>; 4]; 4] = Default::default();

        let mut request_tiles: Vec<Tile> = Vec::new();
        for x in 0..4 {
            for y in 0..4 {
                request_tiles.push(Tile::new((x, y, 2), (x, y)));
            }
        }

        //println!(
        //    "my position tile is {}",
        //    slippy_map_tiles::lat_lon_to_tile(42.473882, -83.473203, 3)
        //);
        (
            MapMaker {
                //TODO: add a new function that handles initializing the array
                tiles: tiles.clone(),
                zoom_in_state: button::State::new(),
                zoom_out_state: button::State::new(),
                cur_coords: (42.473882, -83.473203),
                zoom_level: 4,
            },
            Command::perform(MapMaker::load(request_tiles), MapMaker::process_load),
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
                println!("me zoom in");

                self.zoom_level += 1;
                let request_tiles = self.generate_tiles();
                return Command::perform(MapMaker::load(request_tiles), MapMaker::process_load);
            }
            Message::ZoomOut => {
                println!("me zoom out");
                self.zoom_level -= 1;
                let request_tiles = self.generate_tiles();
                return Command::perform(MapMaker::load(request_tiles), MapMaker::process_load);
            }

            Message::ImageLoadFailed => {
                println!("image load failed");
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
        let content =
            Column::new()
                .padding(20)
                .spacing(20)
                .max_width(2500)
                .push(map_tile::MapTile::new(
                    self.tiles.clone(),
                    &mut self.zoom_in_state,
                    &mut self.zoom_out_state,
                    //https://stackoverflow.com/questions/27895946/expected-fn-item-found-a-different-fn-item-when-working-with-function-pointer
                    zoom_in_spawner as ButtonSpawner,
                    zoom_out_spawner as ButtonSpawner,
                ));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
