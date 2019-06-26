extern crate chrono;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

mod api;
pub mod chrono_utils;
pub mod logging;
mod wallclock;

use chrono::prelude::*;
use std::env;
use wallclock::Wallclock;

fn main() {
    let root_logger = logging::pretty_logger();

    let args: Vec<String> = env::args().collect();

    if args.len() == 3 {
        let start = DateTime::parse_from_str(&args[1], "%Y-%m-%d %H:%M:%S %z")
            .expect("Can't parse start time");
        let end = DateTime::parse_from_str(&args[2], "%Y-%m-%d %H:%M:%S %z")
            .expect("Can't parse end time");
        let wc = Wallclock::new(root_logger.new(o!("wallclock" => "singleton")), start, end);

        let t = wc.go(Utc::now());

        const FMT_NOTZ: &str = "%a %e %T";
        const FMT_TZ: &str = "%a %e %T %Z";
        println!(
            "{} | {} | {} | {} | {}",
            start.format(FMT_TZ),
            chrono_utils::render_hours_mins(t.elapsed),
            t.walltime.format(FMT_NOTZ),
            chrono_utils::render_hours_mins(t.remaining),
            end.format(FMT_TZ)
        );
    } else if args.len() == 1 {
        let addr = "0.0.0.0:8080";
        api::serve(root_logger.new(o!("server" => addr)), addr.to_string());
    } else {
        crit!(root_logger, "Usage");
    }
}
