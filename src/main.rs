mod concierge;
mod ws;

// Listen on every available network interface
pub const SOCKET_ADDR: ([u8; 4], u16) = ([0, 0, 0, 0], 64209);
pub const VERSION: &str = "0.2.0";
pub const MIN_VERSION: &str = "^0.2.0";
pub const SECRET: Option<&str> = None;
pub const SUBPROTOCOL: &str = "ert-concierge";

pub const FS_KEY_HEADER: &str = "x-fs-key";

pub fn min_version_req() -> VersionReq {
    VersionReq::parse(crate::MIN_VERSION).expect("Valid versioning scheme")
}

use std::{
    net::SocketAddr,
    time::Instant,
};

use actix::prelude::*;
use actix_web::{web, middleware, App, Error, HttpRequest, HttpResponse, HttpServer};
use concierge::Concierge;
use uuid::Uuid;
use ws::WsChatSession;
use semver::VersionReq;

/// Entry point for our route
async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<Concierge>>,
) -> Result<HttpResponse, Error> {
    println!("test");
    actix_web_actors::ws::start_with_protocols(
        WsChatSession {
            uuid: Uuid::nil(),
            last_hb: Instant::now(),
            concierge_addr: srv.get_ref().clone(),
        },
        &[SUBPROTOCOL],
        &req,
        stream,
    )
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    //     // Setup the logging
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let server = Concierge::new().start();
    HttpServer::new(move || {
        App::new()
            .data(server.clone())
            .service(web::resource("/").route(web::get().to(|| {
                HttpResponse::Found()
                    .header("LOCATION", "/static/websocket.html")
                    .finish()
            })))
            .wrap(middleware::Logger::default())
            .service(web::resource("/ws").route(web::get().to(ws_index)))
    })
    .bind(SocketAddr::from(SOCKET_ADDR))?
    .run()
    .await
}

// async fn serve() {
//     info!("Starting up the server.");

//     // Wrap the server in an atomic ref-counter, to make it safe to work with in between threads.
//     let concierge = Arc::new(Concierge::new());

//     let addr = SocketAddr::from(SOCKET_ADDR);

//     let ws_route = {
//         let concierge = concierge.clone();
//         warp::get()
//             .and(warp::path("ws"))
//             .and(warp::addr::remote())
//             .and(warp::ws())
//             .map(move |addr: Option<SocketAddr>, ws: warp::ws::Ws| {
//                 debug!("Incoming TCP connection. (ip: {:?})", addr);
//                 let concierge = concierge.clone();
//                 ws.on_upgrade(move |websocket| async move {
//                     concierge.handle_socket_conn(websocket, addr).await
//                 })
//             })
//             .map(|reply| {
//                 warp::reply::with_header(
//                     reply,
//                     header::SEC_WEBSOCKET_PROTOCOL.as_str(),
//                     SUBPROTOCOL,
//                 )
//             })
//     };

//     let fs_download_route = {
//         let concierge = concierge.clone();
//         warp::get()
//             .and(warp::path("fs"))
//             .and(warp::path::param::<String>())
//             .and(warp::path::tail())
//             .and(warp::header::<Uuid>(FS_KEY_HEADER))
//             .and_then(move |name: String, path: Tail, auth: Uuid| {
//                 let concierge = concierge.clone();
//                 async move {
//                     concierge
//                         .fs_conn()
//                         .handle_file_get(name, auth, path.as_str())
//                         .await
//                         .map_err(FsError::rejection)
//                 }
//             })
//     };

//     // Binary upload
//     let fs_upload_route = {
//         let concierge = concierge.clone();
//         warp::put()
//             .and(warp::path("fs"))
//             .and(warp::path::param::<String>())
//             .and(warp::path::tail())
//             .and(warp::header::<Uuid>(FS_KEY_HEADER))
//             // 2mb upload limit
//             .and(warp::body::content_length_limit(1024 * 1024 * 2))
//             .and(warp::body::aggregate())
//             .and_then(move |name: String, tail: Tail, auth: Uuid, stream| {
//                 let concierge = concierge.clone();
//                 async move {
//                     concierge
//                         .fs_conn()
//                         .handle_file_put(name, auth, tail.as_str(), stream)
//                         .await
//                         .map_err(FsError::rejection)
//                 }
//             })
//     };

//     // Form upload
//     let fs_upload_multipart_route = {
//         let concierge = concierge.clone();
//         warp::post()
//             .and(warp::path("fs"))
//             .and(warp::path::param::<String>())
//             .and(warp::path::tail())
//             .and(warp::header::<Uuid>(FS_KEY_HEADER))
//             .and(warp::multipart::form())
//             .and_then(
//                 move |name: String, tail: Tail, auth: Uuid, data: FormData| {
//                     let concierge = concierge.clone();
//                     async move {
//                         concierge
//                             .fs_conn()
//                             .handle_file_put_multipart(name, auth, tail.as_str(), data)
//                             .await
//                             .map_err(FsError::rejection)
//                     }
//                 },
//             )
//     };

//     let fs_delete_route = {
//         warp::delete()
//             .and(warp::path("fs"))
//             .and(warp::path::param::<String>())
//             .and(warp::path::tail())
//             .and(warp::header::<Uuid>(FS_KEY_HEADER))
//             .and_then(move |name: String, tail: Tail, auth: Uuid| {
//                 let concierge = concierge.clone();
//                 async move {
//                     concierge
//                         .fs_conn()
//                         .handle_file_delete(name, auth, tail.as_str())
//                         .await
//                         .map_err(FsError::rejection)
//                 }
//             })
//     };

//     let routes = ws_route
//         .or(fs_download_route.or(fs_delete_route))
//         .or(fs_upload_route.or(fs_upload_multipart_route))
//         .with(
//             warp::cors()
//                 .allow_any_origin()
//                 .allow_methods(&[Method::POST, Method::GET, Method::DELETE])
//                 .allow_header(FS_KEY_HEADER)
//                 .allow_header("*"),
//         );

//     warp::serve(routes)
//         // .tls()
//         // .cert_path("./tls/cert.pem")
//         // .key_path("./tls/key.rsa")
//         .run(addr)
//         .await;
// }
