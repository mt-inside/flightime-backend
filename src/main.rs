extern crate chrono;
extern crate gotham;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use chrono::prelude::*;
use chrono::Duration;
use gotham::state::State;

use slog::Drain;
use std::env;

pub fn time_handler(state: State) -> (State, String) {
    let reply = format!("Hello owlrdn");

    (state, reply)
}
fn scale_duration(lhs: Duration, rhs: f64) -> Duration {
    let secs = lhs.num_seconds() as f64 * rhs;
    Duration::seconds(secs as i64)
}

fn go(
    logger: slog::Logger,
    start: DateTime<FixedOffset>,
    end: DateTime<FixedOffset>,
    now: DateTime<FixedOffset>, // TODO take UTC
) -> chrono::DateTime<FixedOffset> {
    /*
    LHR -> JFK. 8hr flight. -5 tz. leave 10am GMT, arrive 1pm EST. Half-way point = 2pm GMT
    e = 8 - 5 = 3
    r = 3/8
    t = 10 + 4 * 3/8 = 11.5 TICK

    LHR -> SIN. 12h flight. +8 tz. Leave 9pm GMT, arrive 5pm SGT. Half-way point = 3am GMT
    e = 12 + 8 = 20
    r = 20/12
    t = 9 + 6 * 20/12 = 7 TICK

    e= d + tz # elapsed time in our wonky space; time to have added to the takeoff by the end
    r = e / d # time runs at this rate
    t = a + (n - a) * r
    */

    let duration = end.signed_duration_since(start);
    trace!(logger, "duration: {}", duration.num_hours());
    let tzdiff = end.timezone().fix().local_minus_utc() - start.timezone().fix().local_minus_utc();
    trace!(logger, "tzdiff: {}", tzdiff / 60 / 60);
    let wallclock_elapsed = duration
        .checked_add(&Duration::seconds(tzdiff.into()))
        .unwrap();
    let wallclock_rate = wallclock_elapsed.num_hours() as f64 / duration.num_hours() as f64;
    let wallclock = start + scale_duration(now.signed_duration_since(start), wallclock_rate);
    wallclock
}

fn init_logging() -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let formatter = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(formatter).build().fuse();
    let root_logger = slog::Logger::root(drain, o!());
    root_logger
}

fn main() {
    let root_logger = init_logging();

    let args: Vec<String> = env::args().collect();
    let addr = "0.0.0.0:8080";

    // TODO POST to configure

    let start =
        DateTime::parse_from_str(&args[1], "%Y-%m-%d %H:%M:%S %z").expect("Can't parse start time");
    let end =
        DateTime::parse_from_str(&args[2], "%Y-%m-%d %H:%M:%S %z").expect("Can't parse end time");
    let now = Utc::now().with_timezone(&start.timezone());

    let wallclock = go(root_logger, start, end, now);
    const FMT_NOTZ: &str = "%a %e %T";
    const FMT_TZ: &str = "%a %e %T %z";
    println!(
        "{} | {} | {} | {} | {}",
        start.format(FMT_TZ),
        now.signed_duration_since(start).num_hours(),
        wallclock.format(FMT_NOTZ),
        end.signed_duration_since(now).num_hours(),
        end.format(FMT_TZ)
    );

    // println!("Listening for requests at http://{}", addr);
    // gotham::start(addr, || Ok(time_handler))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: i32 = 60 * 60;

    #[test]
    fn go_test_lhr_jfk() {
        let logger = init_logging();

        let start = FixedOffset::east(0 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(10, 0, 0);
        let end = FixedOffset::east(-5 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(13, 0, 0);
        let now = FixedOffset::east(0 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(14, 0, 0);

        let wallclock = go(logger, start, end, now);

        assert_eq!(
            wallclock,
            FixedOffset::east(0 * HOUR)
                .ymd(2019, 6, 14)
                .and_hms(11, 30, 0)
        );
    }

    #[test]
    fn go_test_lhr_sin() {
        let logger = init_logging();

        let start = FixedOffset::east(0 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(21, 0, 0);
        let end = FixedOffset::east(8 * HOUR)
            .ymd(2019, 6, 15)
            .and_hms(17, 0, 0);
        let now = FixedOffset::east(0 * HOUR)
            .ymd(2019, 6, 15)
            .and_hms(3, 0, 0);

        let wallclock = go(logger, start, end, now);

        assert_eq!(
            wallclock,
            FixedOffset::east(0 * HOUR)
                .ymd(2019, 6, 15)
                .and_hms(7, 0, 0)
        );
    }

    #[test]
    fn go_test_pvg_ewr() {
        let logger = init_logging();

        let start = FixedOffset::east(8 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(15, 0, 0); //7am UTC
        let end = FixedOffset::east(-5 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(18, 0, 0); // 11pm UTC (16h)
        let now = FixedOffset::east(8 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(23, 0, 0);

        let wallclock = go(logger, start, end, now);

        assert_eq!(
            wallclock,
            FixedOffset::east(8 * HOUR)
                .ymd(2019, 6, 14)
                .and_hms(16, 30, 0)
        );
    }

    #[test]
    fn go_test_ewr_pvg() {
        let logger = init_logging();

        let start = FixedOffset::east(-5 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(10, 0, 0); //3pm UTC
        let end = FixedOffset::east(8 * HOUR) // 13h tz
            .ymd(2019, 6, 15)
            .and_hms(13, 0, 0); // 5am UTC (14h?)
        let now = FixedOffset::east(-5 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(17, 0, 0);

        let wallclock = go(logger, start, end, now);

        assert_eq!(
            wallclock,
            FixedOffset::east(-5 * HOUR)
                .ymd(2019, 6, 14)
                .and_hms(23, 30, 0)
        );
    }
}
