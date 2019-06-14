extern crate chrono;
extern crate gotham;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use chrono::prelude::*;
use gotham::state::State;

use std::env;

mod wallclock;
use wallclock::Wallclock;

pub mod logging;

pub fn time_handler(state: State) -> (State, String) {
    let reply = format!("Hello owlrdn");

    (state, reply)
}

fn main() {
    let root_logger = logging::pretty_logger();

    let args: Vec<String> = env::args().collect();

    // TODO POST to configure

    let start =
        DateTime::parse_from_str(&args[1], "%Y-%m-%d %H:%M:%S %z").expect("Can't parse start time");
    let end =
        DateTime::parse_from_str(&args[2], "%Y-%m-%d %H:%M:%S %z").expect("Can't parse end time");
    let wc = Wallclock::new(root_logger, start, end);

    let now = Utc::now().with_timezone(&start.timezone());
    let t = wc.go(now);

    const FMT_NOTZ: &str = "%a %e %T";
    const FMT_TZ: &str = "%a %e %T %Z";
    println!(
        "{} | {:.1} | {} | {:.1} | {}",
        start.format(FMT_TZ),
        t.elapsed_s as f32 / 60.0 / 60.0,
        t.walltime.format(FMT_NOTZ),
        t.remaining_s as f32 / 60.0 / 60.0,
        end.format(FMT_TZ)
    );

    //let addr = "0.0.0.0:8080";
    // println!("Listening for requests at http://{}", addr);
    // gotham::start(addr, || Ok(time_handler))
}
