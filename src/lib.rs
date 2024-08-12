pub mod common;

pub mod routes_assets;

pub mod routes_account;
pub mod routes_index;

pub mod state;

pub mod utils {
    pub mod date_config;
    pub mod q_body;
    pub mod jwt;
    pub mod message;
    pub mod date_option;
    pub mod db;
    pub mod cookie;
}
pub mod auth {
    pub mod handlers;
    pub mod models;
    // pub mod repository;
    pub mod middleware;
    // pub mod views;
}
pub mod profile {
    pub mod handlers;
    pub mod models;
    // pub mod repository;
    // pub mod views;
}

