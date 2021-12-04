pub mod tile_manager {
    use futures::future::join_all;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[derive(Clone, Debug)]
    pub enum TileState {
        NotLoaded,
        Loading,
        Loaded,
    }
    #[derive(Clone, Debug)]
    pub struct Tile {
        pub target_url: (u32, u32, u32),
        pub image: Vec<u8>,
        pub state: TileState,
    }

    impl Tile {
        pub fn new(target_url: &(u32, u32, u32)) -> Self {
            Self {
                target_url: *target_url,
                image: Vec::new(),
                state: TileState::NotLoaded,
            }
        }
    }
    impl Default for Tile {
        fn default() -> Self {
            Self {
                target_url: (0, 0, 0),
                image: Vec::new(),
                state: TileState::NotLoaded,
            }
        }
    }

    pub struct TileManager {
        tile_dict: HashMap<(u32, u32, u32), Tile>,
        pub client: std::sync::Arc<reqwest::Client>,
        load_queue: Vec<(u32, u32, u32)>,
    }

    impl Default for TileManager {
        fn default() -> Self {
            Self {
                tile_dict: Default::default(),
                client: std::sync::Arc::new(reqwest::Client::new()),
                load_queue: Default::default(),
            }
        }
    }

    impl TileManager {
        pub fn new() -> Self {
            Self {
                tile_dict: Default::default(),
                client: std::sync::Arc::new(reqwest::Client::new()),
                load_queue: Default::default(),
            }
        }

        pub fn get_tile(&mut self, coords: &(u32, u32, u32)) -> Tile {
            //x, y, z format
            match self.tile_dict.get(coords) {
                Some(tile) => {
                    match tile.state {
                        TileState::Loaded => return tile.clone(),
                        TileState::Loading => return tile.clone(),
                        TileState::NotLoaded => return tile.clone(),
                    }
                }
                None => {
                    let new_tile = Tile::new(coords);
                    self.tile_dict.insert(*coords, new_tile.clone());
                    return new_tile;
                }
            }
        }

        pub fn queue_tile_load(&mut self, coords: (u32, u32, u32)) {
            self.load_queue.push(coords);

        }

        async fn load_tile(
            mut request_tile: Tile,
            client: Arc<reqwest::Client>,
        ) -> Result<Tile, reqwest::Error> {
            let (x, y, z) = request_tile.target_url;
            let request = client
                .get(format!(
                    "https://stamen-tiles.a.ssl.fastly.net/terrain/{}/{}/{}.png",
                    z, x, y
                ))
                .send()
                .await;

            if let Ok(resp) = request {
                let bytes_req = resp.bytes().await;
                if let Ok(bytes) = bytes_req {
                    request_tile.image = bytes.to_vec();
                    request_tile.state = TileState::Loaded;
                }
            }
            Ok(request_tile)
        }
        pub fn ingest_loaded_tiles(&mut self, mut new_tiles: Vec<Tile>) {
            for mut tile in new_tiles {
                tile.state = TileState::Loaded;
                self.tile_dict.insert(tile.target_url, tile);
            }
        }

        pub fn generate_async_load(&mut self) ->impl futures::Future<Output=Option<Vec<Tile>>> {

            let load_queue =  self.load_queue.clone();
            self.load_queue.clear();
            TileManager::load_tiles(self.client.clone(),load_queue)
        }

        pub async fn load_tiles(
            client: Arc<reqwest::Client>,
            load_coords: Vec<(u32,u32,u32)>,
        ) -> Option<Vec<Tile>> {
            
            log::info!("me loading {} tiles", load_coords.len());
            let mut return_tiles: Vec<Tile> = load_coords.iter().map(|coord| Tile::new(coord)).collect::<Vec<Tile>>();

            let tile_futures = return_tiles
                .iter_mut()
                .map(|tile| TileManager::load_tile(tile.clone(), client.clone()));

            let tile_results = join_all(tile_futures).await;

            for tile_result in tile_results {
                if let Ok(tile) = tile_result {
                    return_tiles.push(tile);
                }
            }
            Some(return_tiles)
        }
    }
}
