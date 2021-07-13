use crate::{
    utils::find_zk_root,
    watch,
};
use mdbook::MDBook;
use mdbook_katex::KatexProcessor;
use mdbook_backlinks::Backlinks;
use mdbook_wikilink::WikiLinks;
use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use mdbook::errors::*;
use mdbook::utils;
use mdbook::utils::fs::get_404_output_file;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use failure::{Error, err_msg};
use tokio::sync::broadcast;
use warp::ws::Message;
use warp::Filter;
use toml;

/// The HTTP endpoint for the websocket used to trigger reloads when a file changes.
const LIVE_RELOAD_ENDPOINT: &str = "__livereload";

pub fn serve() -> Result<(), Error> {
    let root = find_zk_root().ok_or(err_msg("Could not find the root of your Zettelkasten"))?;

    let mut zk = match MDBook::load(root) {
        Ok(val) => val,
        Err(e) => return Err(err_msg(e.to_string())),
    };

    zk.with_preprocessor(KatexProcessor);
    zk.with_preprocessor(Backlinks);
    zk.with_preprocessor(WikiLinks);

    let port = "3000";
    let hostname = "localhost";
    // let open_browser = false;

    let address = format!("{}:{}", hostname, port);

    let livereload_url = format!("ws://{}/{}", address, LIVE_RELOAD_ENDPOINT);
    let update_config = |book: &mut MDBook| {
        book.config
            .set("output.html.livereload-url", &livereload_url)
            .expect("livereload-url update failed");
        // Override site-url for local serving of the 404 file
        book.config.set("output.html.site-url", "/").unwrap();
    };
    update_config(&mut zk);

    match zk.build() {
        Ok(_) => {},
        Err(e) => return Err(err_msg(e.to_string())),
    }

    let sockaddr: SocketAddr = address
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| err_msg("no address found for ".to_string() + &address))?;
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
    println!("Serving on: {}", serving_url);

    /* if open_browser {
        open(serving_url);
    } */

    watch::trigger_on_change(&zk, move |paths, book_dir| {
        println!("Files changed: {:?}", paths);
        println!("Building book...");

        // FIXME: This area is really ugly because we need to re-set livereload :(
        let result = MDBook::load(&book_dir).and_then(|mut b| {
            update_config(&mut b);
            b.build()
        });

        if let Err(e) = result {
            println!("Unable to load the book");
            utils::log_backtrace(&e);
        } else {
            let _ = tx.send(Message::text("reload"));
        }
    });

    let _ = thread_handle.join();

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
                println!("websocket got connection");
                if let Ok(m) = rx.recv().await {
                    println!("notify of reload");
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
        println!("Unable to serve: {}", panic_info);
        std::process::exit(1);
    }));

    warp::serve(routes).run(address).await;
}
