use std::{env, io};

use actix_web::{web, App, HttpServer};
use log::info;
use webdav_handler::actix::*;
use webdav_handler::{fakels::FakeLs, DavConfig, DavHandler};

use vfs::AliyunDriveFileSystem;

mod aliyundrive;
mod vfs;

pub async fn dav_handler(req: DavRequest, davhandler: web::Data<DavHandler>) -> DavResponse {
    if let Some(prefix) = req.prefix() {
        let config = DavConfig::new().strip_prefix(prefix);
        davhandler.handle_with(config, req.request).await.into()
    } else {
        davhandler.handle(req.request).await.into()
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "aliyundrive_webdav=info");
    }
    pretty_env_logger::init();
    let addr = "127.0.0.1:4918";

    let fs = AliyunDriveFileSystem::new("".to_string()).await;
    let dav_server = DavHandler::builder()
        .filesystem(Box::new(fs))
        .locksystem(FakeLs::new())
        .build_handler();

    info!("listening on {}", addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(dav_server.clone()))
            .service(web::resource("/{tail:.*}").to(dav_handler))
    })
    .bind(addr)?
    .run()
    .await
}
