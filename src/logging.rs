use slog::Drain;
use slog_term::TestStdoutWriter;

pub fn pretty_logger() -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let formatter = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(formatter).build().fuse();
    let root_logger = slog::Logger::root(drain, o!());
    root_logger
}

pub fn test_logger() -> slog::Logger {
    let decorator = slog_term::PlainSyncDecorator::new(TestStdoutWriter);
    let formatter = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(formatter).build().fuse();
    let root_logger = slog::Logger::root(drain, o!());
    root_logger
}
