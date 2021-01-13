use crate::decode;
use crate::nodes::Node;
use crate::protocols::Protocol;
use crate::rpc::{Error, Request, Response};
use anyhow::{bail, Result};
use futures::{FutureExt, StreamExt};
use strum::VariantNames;

#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use structopt::StructOpt;
    #[derive(Debug, StructOpt)]
    #[structopt(about = "Serve an executor using HTTP, WebSockets, or Standard I/O")]
    pub struct Args {
        /// Transport protocol to use (defaults to `stdio`)
        #[structopt(long, env = "EXECUTA_PROTOCOL", possible_values = Protocol::VARIANTS, case_insensitive = true)]
        protocol: Option<Protocol>,

        /// Address to listen on (HTTP and Websockets only, defaults to "127.0.0.1")
        #[structopt(short, long, env = "EXECUTA_ADDRESS")]
        address: Option<String>,

        /// Port to listen on (HTTP and Websockets only, defaults to 9000)
        #[structopt(short, long, env = "EXECUTA_PORT")]
        port: Option<u16>,
    }

    pub async fn serve(args: Args) -> Result<Node> {
        let Args {
            protocol,
            address,
            port,
        } = args;

        let (join_handle, ..) = super::serve(protocol, address, port)?;
        join_handle.await?;

        Ok(Node::Null)
    }
}

pub fn serve(
    protocol: Option<Protocol>,
    address: Option<String>,
    port: Option<u16>,
) -> Result<(
    tokio::task::JoinHandle<()>,
    tokio::sync::oneshot::Sender<()>,
)> {
    let protocol = protocol.unwrap_or(if cfg!(feature = "serve-stdio") {
        Protocol::Stdio
    } else if cfg!(feature = "serve-http") {
        Protocol::Http
    } else if cfg!(feature = "serve-ws") {
        Protocol::Ws
    } else {
        bail!("There are no serve-* features enabled")
    });

    let address: std::net::IpAddr = address.unwrap_or_else(|| "127.0.0.1".to_string()).parse()?;

    let port = port.unwrap_or(9000);

    match protocol {
        Protocol::Stdio => todo!(),
        Protocol::Http | Protocol::Ws => {
            use warp::Filter;

            let post = warp::path::end()
                .and(warp::post())
                .and(warp::body::json::<Request>())
                .map(post_handler);

            let post_wrap = warp::path::param()
                .and(warp::post())
                .and(warp::body::json::<serde_json::Value>())
                .map(post_wrap_handler);

            let ws = warp::path::end().and(warp::ws()).map(ws_handler);

            let cors = warp::cors()
                .allow_any_origin()
                .allow_headers(vec![
                    "Content-Type",
                    "Referer",
                    "Origin",
                    "Access-Control-Allow-Origin",
                ])
                .allow_methods(&[warp::http::Method::GET, warp::http::Method::POST])
                .max_age(24 * 60 * 60);

            let routes = post
                .or(post_wrap)
                .or(ws)
                .with(cors)
                .recover(rejection_handler);

            use tokio::sync::oneshot;
            let (sender, reciever) = oneshot::channel::<()>();

            let (_, server) =
                warp::serve(routes).bind_with_graceful_shutdown((address, port), async {
                    reciever.await.ok();
                });

            let join_handle = tokio::task::spawn(server);

            Ok((join_handle, sender))
        }
    }
}

fn post_handler(request: Request) -> impl warp::Reply {
    let response = respond(request);
    warp::reply::json(&response)
}

fn post_wrap_handler(method: String, params: serde_json::Value) -> impl warp::Reply {
    use warp::http::StatusCode;
    use warp::reply;

    // Wrap the method and parameters into a request
    let request = serde_json::from_value::<Request>(serde_json::json!({
        "method": method,
        "params": params
    }));
    let request = match request {
        Ok(request) => request,
        Err(error) => {
            return reply::with_status(
                reply::json(&serde_json::json!({
                    "message": error.to_string()
                })),
                StatusCode::BAD_REQUEST,
            )
        }
    };

    // Unwrap the response into results or error message
    let Response { result, error, .. } = respond(request);
    match result {
        Some(result) => reply::with_status(reply::json(&result), StatusCode::OK),
        None => match error {
            Some(error) => reply::with_status(reply::json(&error), StatusCode::BAD_REQUEST),
            None => reply::with_status(
                reply::json(&serde_json::json!({
                    "message": "Response had neither a result nor an error"
                })),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        },
    }
}

fn ws_handler(ws: warp::ws::Ws) -> impl warp::Reply {
    ws.on_upgrade(|socket| {
        // TODO Currently just echos
        let (tx, rx) = socket.split();
        rx.forward(tx).map(|result| {
            if let Err(e) = result {
                eprintln!("websocket error: {:?}", e);
            }
        })
    })
}

/// Handle a rejection by converting into a JSON-RPC response
///
/// The above handlers can not handle all errors, in particular, they do not
/// handle JSON parsing errors (which are rejected by the `warp::body::json` filter).
/// This therefore ensures that any request expecting a JSON-RPC response, will get
/// a JSON-RPC response (in these cases containing and error code and message).
async fn rejection_handler(
    rejection: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let code: i16;
    let message;

    if let Some(error) = rejection.find::<warp::filters::body::BodyDeserializeError>() {
        code = -32700;
        message = format!("{}", error);
    } else if rejection.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = -32700;
        message = "Method not found".to_string();
    } else {
        code = -32000;
        message = "Internal server error".to_string();
    }

    Ok(warp::reply::json(&Response {
        error: Some(Error { code, message }),
        ..Default::default()
    }))
}

fn respond(request: Request) -> Response {
    let id = Request::id(&request);
    let result = match request {
        Request::Decode(request) => decode::rpc::decode(request.params),
    };
    match result {
        Ok(node) => Response::new(id, Some(node), None),
        Err(error) => Response::new(id, None, Some(error)),
    }
}
