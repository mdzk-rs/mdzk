use crate::renderer::HtmlMdzk;
use crate::{cmd::watch, load_zk};
use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use mdbook::{
    errors::*,
    renderer::{HtmlHandlebars, MarkdownRenderer},
    utils,
    utils::fs::get_404_output_file,
    MDBook,
};
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use tokio::sync::broadcast;
use toml;
use warp::ws::Message;
use warp::Filter;

/// The HTTP endpoint for the websocket used to trigger reloads when a file changes.
const LIVE_RELOAD_ENDPOINT: &str = "__livereload";

pub fn serve(dir: Option<PathBuf>, port: i32, bind: String, renderer: String) -> Result<(), Error> {
    let mut zk = load_zk(dir)?;

    let address = format!("{}:{}", bind, port.to_string());

    let livereload_url = format!("ws://{}/{}", address, LIVE_RELOAD_ENDPOINT);
    update_config(&mut zk, &livereload_url)?;

    match renderer.as_str() {
        "markdown" => zk.execute_build_process(&MarkdownRenderer)?,
        "mdzk" => zk.execute_build_process(&HtmlMdzk)?,
        _ => zk.execute_build_process(&HtmlHandlebars)?,
    }

    let sockaddr: SocketAddr = address
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| Error::msg("no address found for ".to_string() + &address))?;
    let build_dir = zk.build_dir_for("html");
    let input_404 = zk
        .config
        .get("output.html.input-404")
        .map(toml::Value::as_str)
        .and_then(std::convert::identity) // flatten
        .map(ToString::to_string);
    let file_404 = get_404_output_file(&input_404);

    // A channel used to broadcast to any websockets to reload when a file changes.
    let (tx, _rx) = tokio::sync::broadcast::channel::<Message>(100);

    let reload_tx = tx.clone();
    let thread_handle = std::thread::spawn(move || {
        serve_zk(build_dir, sockaddr, reload_tx, &file_404);
    });

    let serving_url = format!("http://{}", address);
    info!("Serving on: {}", serving_url);

    /* if open_browser {
        open(serving_url);
    } */

    watch::trigger_on_change(&zk, move |paths, book_dir| {
        info!("Files changed: {:?}", paths);
        info!("Building book...");

        let mut new_zk = load_zk(Some(book_dir.to_path_buf())).unwrap();
        update_config(&mut new_zk, &livereload_url).unwrap();
        let result = match renderer.as_str() {
            "markdown" => new_zk.execute_build_process(&MarkdownRenderer),
            "mdzk" => new_zk.execute_build_process(&HtmlMdzk),
            _ => new_zk.execute_build_process(&HtmlHandlebars),
        };

        if let Err(e) = result {
            error!("Unable to load the book");
            utils::log_backtrace(&e);
        } else {
            let _ = tx.send(Message::text("reload"));
        }
    });

    let _ = thread_handle.join();

    Ok(())
}

fn update_config(book: &mut MDBook, livereload_url: &str) -> Result<()> {
    // Override site-url for local serving of the 404 file
    book.config.set("output.html.site-url", "/")?;
    book.config
        .set("output.html.livereload-url", &livereload_url)?;
    Ok(())
}

#[tokio::main]
async fn serve_zk(
    build_dir: PathBuf,
    address: SocketAddr,
    reload_tx: broadcast::Sender<Message>,
    file_404: &str,
) {
    // A warp Filter which captures `reload_tx` and provides an `rx` copy to
    // receive reload messages.
    let sender = warp::any().map(move || reload_tx.subscribe());

    // A warp Filter to handle the livereload endpoint. This upgrades to a
    // websocket, and then waits for any filesystem change notifications, and
    // relays them over the websocket.
    let livereload = warp::path(LIVE_RELOAD_ENDPOINT)
        .and(warp::ws())
        .and(sender)
        .map(|ws: warp::ws::Ws, mut rx: broadcast::Receiver<Message>| {
            ws.on_upgrade(move |ws| async move {
                let (mut user_ws_tx, _user_ws_rx) = ws.split();
                trace!("websocket got connection");
                if let Ok(m) = rx.recv().await {
                    trace!("notify of reload");
                    let _ = user_ws_tx.send(m).await;
                }
            })
        });
    // A warp Filter that serves from the filesystem.
    let book_route = warp::fs::dir(build_dir.clone());
    // The fallback route for 404 errors
    let fallback_route = warp::fs::file(build_dir.join(file_404))
        .map(|reply| warp::reply::with_status(reply, warp::http::StatusCode::NOT_FOUND));
    let routes = livereload.or(book_route).or(fallback_route);

    std::panic::set_hook(Box::new(move |panic_info| {
        // exit if serve panics
        error!("Unable to serve: {}", panic_info);
        std::process::exit(1);
    }));

    warp::serve(routes).run(address).await;
}
