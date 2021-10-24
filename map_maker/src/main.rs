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
    executor, slider, Alignment, Application, Column, Command, Container, Element, Length, Sandbox,
    Settings, Slider, Text,
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

#[derive(Debug)]
enum Message {
    LoadedImage(Result<(u32, Vec<u8>), MyError>),
}

#[derive(Debug, Error)]
enum MyError {
    #[error("api error")]
    APIError,
    #[error("https error")]
    HttpsError(#[from] reqwest::Error),
}

impl MapMaker {
    async fn load() -> Result<(u32, Vec<u8>), MyError> {
        println!("loading");
        let resp = reqwest::get("https://stamen-tiles.a.ssl.fastly.net/terrain/2/1/3.png")
            .await?
            .bytes()
            .await?
            .to_vec();
        println!("found my stuff");
        Ok((0, resp))
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
        (
            MapMaker {
                //TODO: add a new function that handles initializing the array
                tiles: tiles.clone(),
            },
            Command::perform(MapMaker::load(), Message::LoadedImage),
        )
    }

    fn title(&self) -> String {
        String::from("MapMaker")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadedImage(resp) => {
                if let Ok(tile_tuple) = resp {
                    self.tiles[tile_tuple.0 as usize][tile_tuple.0 as usize] =
                        (tile_tuple.1).clone();
                    self.tiles[0][1] = (tile_tuple.1).clone();
                    self.tiles[0][2] = (tile_tuple.1).clone();
                    self.tiles[(3) as usize][(3) as usize] = tile_tuple.1;
                }
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let content = Column::new()
            .padding(20)
            .spacing(20)
            .max_width(2500)
            .push(map_tile::MapTile::new(self.tiles.clone()));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
