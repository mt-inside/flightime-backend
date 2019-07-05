# Flightime
NB: This is super-pre-alpha

Flightime calculates a "timezone-fluid" "adjusted" "walltime".
Imagine you leave at 1pm, fly for 5 hours, but land at 3pm because you gain three hours from the timezone difference.
Flightime will calculate a "wallclock" time which counts from 1pm to 3pm more slowly, for the whole 5 hour duration of the flight.
Thus, you can look at the "wallclock", which will run faster or slower than real-time, but not jump through a discontinuity.

The system clock will be used to calculate the walltime.
If the system clock is outside the provided start and end times, the calculted results will be weird.

Flightime has a CLI mode for convenience, which you can run in a loop or something.
However it also has a JSON/HTTP endpoint, the idea being that it would provide the system tray time for a desktop environment like [i3wm](https://i3wm.org/).

## tl;dr
`$ cargo run <start time> <end time>`
`$ cargo run "2019-07-04 20:10:00 -0400" "2019-07-05 10:10:00 +0100"`

## Building
### Prerequisites
Flightime is written in *Rust*.
You'll need [the Rust toolchain and package manager](https://www.rust-lang.org/tools/install).

### Compilation
Build with `cargo build`.
This will give you a debug binary at `target/debug/flightime-backend`.

### Running
Run the binary built above, or use `cargo run`.

### Server
This project is meant to be a "backend"; an API endpoint.
Run with no arguments, flightime will run as a daemon, listening on port `8080`.
Send an HTTP `POST` with a JSON body containing fields:

* `start`: the start time of the flight, with timezone, conforming to `%Y-%m-%d %H:%M:%S %z`
* `end`: the end time of the flight, with timezone, conforming to `%Y-%m-%d %H:%M:%S %z`

The returned JSON object will tell you the flight time elaspsed and remaining in seconds, plus the "walltime".

### CLI
Flightime also has a run-once CLI mode, where the start and end time can be provided as command-line arguments, in the same format as for the JSON endpoint.

# TODO
error handling
