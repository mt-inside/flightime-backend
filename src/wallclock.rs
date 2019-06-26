use chrono::prelude::*;
use chrono::Duration;

use crate::chrono_utils;

pub struct Wallclock {
    logger: slog::Logger,
    start: DateTime<FixedOffset>,
    end: DateTime<FixedOffset>,
}

pub struct Walltime {
    pub elapsed: Duration,
    pub walltime: DateTime<FixedOffset>,
    pub remaining: Duration,
}

impl Wallclock {
    pub fn new(
        logger: slog::Logger,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
    ) -> Self {
        Wallclock { logger, start, end }
    }

    pub fn go(&self, utc_now: DateTime<Utc>) -> Walltime {
        let now = utc_now.with_timezone(&self.start.timezone());

        let elapsed = now.signed_duration_since(self.start);
        let remaining = self.end.signed_duration_since(now);

        if now < self.start {
            return Walltime {
                elapsed,
                walltime: now,
                remaining,
            };
        }
        if now > self.end {
            return Walltime {
                elapsed,
                walltime: now,
                remaining,
            };
        }

        let duration = self.end.signed_duration_since(self.start);
        trace!(
            self.logger,
            "duration: {}",
            chrono_utils::render_hours_mins(duration)
        );
        let tzdiff = self.end.timezone().fix().local_minus_utc()
            - self.start.timezone().fix().local_minus_utc();
        trace!(self.logger, "tzdiff: {:.1}", tzdiff as f32 / 60.0 / 60.0);
        let wallclock_elapsed = duration
            .checked_add(&Duration::seconds(tzdiff.into()))
            .unwrap();
        trace!(
            self.logger,
            "apparent length: {}",
            chrono_utils::render_hours_mins(wallclock_elapsed)
        );
        let wallclock_rate = wallclock_elapsed.num_hours() as f64 / duration.num_hours() as f64;
        let walltime =
            self.start + scale_duration(now.signed_duration_since(self.start), wallclock_rate);
        Walltime {
            elapsed,
            walltime,
            remaining,
        }
    }
}

fn scale_duration(lhs: Duration, rhs: f64) -> Duration {
    let secs = lhs.num_seconds() as f64 * rhs;
    Duration::seconds(secs as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::test_logger;

    const HOUR: i32 = 60 * 60;

    /*
    "algorithm":
    e= d + tz # elapsed time in our wonky space; time to have added to the takeoff by the end
    r = e / d # time runs at this rate
    t = a + (n - a) * r
    */

    #[test]
    fn go_test_lhr_jfk() {
        /*
        LHR -> JFK. 8hr flight. -5 tz. leave 10am GMT, arrive 1pm EST. Half-way point = 2pm GMT
        e = 8 - 5 = 3
        r = 3/8
        t = 10 + 4 * 3/8 = 11.5 TICK
        */
        let logger = test_logger();
        let start = FixedOffset::east(0 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(10, 0, 0);
        let end = FixedOffset::east(-5 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(13, 0, 0);
        let wc = Wallclock::new(logger, start, end);

        let now = FixedOffset::east(0 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(14, 0, 0);
        let t = wc.go(now.with_timezone(&Utc));

        assert_eq!(
            t.walltime,
            FixedOffset::east(0 * HOUR)
                .ymd(2019, 6, 14)
                .and_hms(11, 30, 0)
        );
    }

    #[test]
    fn go_test_lhr_sin() {
        /*
        LHR -> SIN. 12h flight. +8 tz. Leave 9pm GMT, arrive 5pm SGT. Half-way point = 3am GMT
        e = 12 + 8 = 20
        r = 20/12
        t = 9 + 6 * 20/12 = 7 TICK
        */
        let logger = test_logger();
        let start = FixedOffset::east(0 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(21, 0, 0);
        let end = FixedOffset::east(8 * HOUR)
            .ymd(2019, 6, 15)
            .and_hms(17, 0, 0);
        let wc = Wallclock::new(logger, start, end);

        let now = FixedOffset::east(0 * HOUR)
            .ymd(2019, 6, 15)
            .and_hms(3, 0, 0);
        let t = wc.go(now.with_timezone(&Utc));

        assert_eq!(
            t.walltime,
            FixedOffset::east(0 * HOUR)
                .ymd(2019, 6, 15)
                .and_hms(7, 0, 0)
        );
    }

    #[test]
    fn go_test_pvg_ewr() {
        let logger = test_logger();
        let start = FixedOffset::east(8 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(15, 0, 0); //7am UTC
        let end = FixedOffset::east(-5 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(18, 0, 0); // 11pm UTC (16h)
        let wc = Wallclock::new(logger, start, end);

        let now = FixedOffset::east(8 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(23, 0, 0);
        let t = wc.go(now.with_timezone(&Utc));

        assert_eq!(
            t.walltime,
            FixedOffset::east(8 * HOUR)
                .ymd(2019, 6, 14)
                .and_hms(16, 30, 0)
        );
    }

    #[test]
    fn go_test_ewr_pvg() {
        let logger = test_logger();
        let start = FixedOffset::east(-5 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(10, 0, 0); //3pm UTC
        let end = FixedOffset::east(8 * HOUR) // 13h tz
            .ymd(2019, 6, 15)
            .and_hms(13, 0, 0); // 5am UTC (14h?)
        let wc = Wallclock::new(logger, start, end);

        let now = FixedOffset::east(-5 * HOUR)
            .ymd(2019, 6, 14)
            .and_hms(17, 0, 0);
        let t = wc.go(now.with_timezone(&Utc));

        assert_eq!(
            t.walltime,
            FixedOffset::east(-5 * HOUR)
                .ymd(2019, 6, 14)
                .and_hms(23, 30, 0)
        );
    }
}
