use chrono::prelude::*;
use futures::{future, Future, Stream};
use gotham::handler::HandlerFuture;
use gotham::helpers::http::response::create_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::single_middleware;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};
use hyper::{Body, StatusCode};

use crate::wallclock::Wallclock;
use std::sync::{Arc, Mutex};

#[derive(Clone, StateData)]
struct ReqState {
    logger: slog::Logger,
    wc: Arc<Mutex<Option<Wallclock>>>,
}
impl ReqState {
    fn new(logger: slog::Logger) -> Self {
        ReqState {
            logger,
            wc: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Deserialize)]
struct WallclockConfig {
    start: String,
    end: String,
}
#[derive(Serialize)]
struct WallclockRender {
    elapsed_s: i64,
    walltime: DateTime<FixedOffset>,
    remaining_s: i64,
}

pub fn serve(logger: slog::Logger, addr: String) {
    warn!(logger, "Listening for requests at http://{}", addr);
    let r = make_router(ReqState::new(logger));

    gotham::start(addr, r);
}

fn make_router(state: ReqState) -> Router {
    let middleware = StateMiddleware::new(state);
    let pipeline = single_middleware(middleware);
    let (chain, pipelines) = single_pipeline(pipeline);

    build_router(chain, pipelines, |route| {
        route.associate("/", |a| {
            a.get().to(get_handler);
            a.post().to(post_handler);
        });
    })
}

fn post_handler(mut state: State) -> Box<HandlerFuture> {
    let f = Body::take_from(&mut state).concat2().then(|b| {
        let s = state.borrow::<ReqState>();
        let logger = &s.logger;

        let body = b.unwrap();
        let content = String::from_utf8(body.to_vec()).unwrap();
        let config: WallclockConfig = serde_json::from_str(&content).unwrap();

        let start = DateTime::parse_from_str(&config.start, "%Y-%m-%d %H:%M:%S %z")
            .expect("Can't parse start time");
        let end = DateTime::parse_from_str(&config.end, "%Y-%m-%d %H:%M:%S %z")
            .expect("Can't parse end time");
        let wc = Wallclock::new(logger.new(o!("wallclock" => "singleton")), start, end);
        *s.wc.lock().unwrap() = Some(wc);

        let response = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN, "ok");
        future::ok((state, response))
    });
    Box::new(f)
}

fn get_handler(state: State) -> Box<HandlerFuture> {
    let response = {
        let s = state.borrow::<ReqState>();
        let wc = &*s.wc.lock().unwrap();
        match wc {
            Some(wc) => {
                let t = wc.go(Utc::now());
                let r = WallclockRender {
                    elapsed_s: t.elapsed.num_seconds(),
                    walltime: t.walltime,
                    remaining_s: t.remaining.num_seconds(),
                };

                create_response(
                    &state,
                    StatusCode::OK,
                    mime::APPLICATION_JSON,
                    serde_json::to_string(&r).expect("Serialisation error"),
                )
            }
            None => create_response(
                &state,
                StatusCode::NOT_FOUND,
                mime::TEXT_PLAIN,
                "not configured",
            ),
        }
    };

    Box::new(future::ok((state, response)))
}
