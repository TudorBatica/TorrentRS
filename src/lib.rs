pub mod config;

pub mod coordinator {
    pub mod choker;
    pub mod ipc;
    pub mod task;
}

pub mod core_models {
    pub mod entities;
    pub mod events;
}

pub mod data_collector;
pub mod dependency_provider;
pub mod file_provider;
pub mod mocks;

pub mod p2p {
    pub mod conn;
    pub mod handlers;
    pub mod state;
    pub mod task;
}

pub mod piece_picker;
pub mod torrent_parser;
pub mod tracker;







