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
}

#[derive(Clone, Debug)]
struct Tile {
    target_url: (u32, u32, u32),
    dest_tile: (u32, u32),
    image: Vec<u8>,
}
impl Tile {
    fn new(target_url: (u32, u32, u32), dest_tile: (u32, u32)) -> Self {
        Self {
            target_url,
            dest_tile,
            image: Vec::new(),
        }
    }
}
#[derive(Debug)]
enum Message {
    LoadedImage(Result<Vec<Tile>, MyError>),
    ButtonPressed(),
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
        (
            MapMaker {
                //TODO: add a new function that handles initializing the array
                tiles: tiles.clone(),
            },
            Command::perform(MapMaker::load(request_tiles), Message::LoadedImage),
        )
    }

    fn title(&self) -> String {
        String::from("MapMaker")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadedImage(resp) => {
                if let Ok(tiles) = resp {
                    for tile in tiles {
                        let (x, y) = tile.dest_tile;
                        self.tiles[x as usize][y as usize] = tile.image;
                    }
                }
            }
            Message::ButtonPressed() => {
                println!("me button was pressed");
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let mut button_state = button::State::new();
        let content =
            Column::new()
                .padding(20)
                .spacing(20)
                .max_width(2500)
                .push(map_tile::MapTile::new(
                    self.tiles.clone(),
                    &mut button_state,
                    |state| {
                        Button::new(state, Text::new("Press Me!")).on_press(Message::ButtonPressed)
                    },
                ));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
