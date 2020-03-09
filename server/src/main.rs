#![feature(proc_macro_hygiene, decl_macro)]
#![feature(drain_filter)]

#[macro_use]
extern crate rocket;

use crate::api::auth::SessionID;
use log::{error, info, Level};
use rocket::response;
use rocket::Config;
use rocket_contrib::serve::StaticFiles;
use serde::Serialize;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

mod api;
mod controller;
mod database;
mod ingame;
mod notify;

pub static SHOULD_TERMINATE: AtomicBool = AtomicBool::new(false);

#[derive(Serialize)]
pub struct PlayerData {
    name: String,
    // TODO typed roles
    role: Option<String>,
}

pub struct SessionData {
    id: SessionID,
    created: SystemTime,
    active: bool,
    settings: Option<String>,
}

#[get("/")]
fn start_get() -> Option<response::NamedFile> {
    response::NamedFile::open("../webapp/dist/start.html").ok()
}

fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(Level::Info).unwrap();

    ctrlc::set_handler(move || {
        SHOULD_TERMINATE.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    info!("Starting server...");
    let addr: SocketAddr = ([127, 0, 0, 1], 3030).into();
    info!("reach under {:?}", addr);

    let static_path = std::fs::canonicalize(Path::new("../webapp/dist"))?;
    info!("Hosting static files from {:?}", static_path);
    let static_files = StaticFiles::from(static_path);

    info!("Opening database...");

    let db = match database::Database::open("test.sqlite".as_ref()) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open database-connection: {}", e);
            return Err(std::io::Error::from(std::io::ErrorKind::NotConnected));
        }
    };

    info!("Starting WebSocket Service...");
    let mut ws_addr = addr.clone();
    ws_addr.set_port(3031);
    let notifier = notify::start(ws_addr)?;

    let mut config = Config::development();
    config.set_port(3030);
    config.set_workers(4);
    config.set_address("0.0.0.0");

    let mut rocket = rocket::custom(config)
        .manage(db)
        .manage(notifier)
        .mount("/", routes![start_get])
        .mount("/static", static_files);

    rocket = ingame::mount_ingame_pages(rocket);
    rocket = controller::mount_controller_pages(rocket);
    rocket = api::mount_current_api_routes(rocket);

    rocket.launch();
    Err(std::io::ErrorKind::Interrupted.into())
}
