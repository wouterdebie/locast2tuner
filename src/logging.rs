use crate::config;
use slog::*;
use slog_async::Async;
use slog_syslog::{Facility, SyslogBuilder};
use slog_term::{FullFormat, PlainDecorator, TermDecorator};
use std::fs::OpenOptions;
use std::sync::Arc;
pub fn logger(log_level: Level, conf: &Arc<config::Config>) -> Logger {
    let term_drain = match &conf.quiet {
        true => None,
        false => Some(
            LevelFilter::new(
                FullFormat::new(TermDecorator::new().build()).build().fuse(),
                log_level,
            )
            .fuse(),
        ),
    };

    let file_drain = match &conf.logfile {
        Some(log_path) => {
            let file = match OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(log_path)
            {
                Ok(f) => f,
                Err(e) => {
                    println!("Unable to open log file '{}'!", log_path);
                    panic!("{}", e);
                }
            };

            Some(
                LevelFilter::new(
                    FullFormat::new(PlainDecorator::new(file)).build().fuse(),
                    log_level,
                )
                .fuse(),
            )
        }
        None => None,
    };

    let syslog_drain = match &conf.syslog {
        true => Some(
            match SyslogBuilder::new()
                .facility(Facility::LOG_USER)
                .level(log_level)
                .unix("/var/run/syslog")
                .start()
            {
                Ok(d) => d,
                Err(e) => {
                    panic!("Failed to start syslog on `/var/run/syslog`. Error {:?}", e)
                }
            }
            .fuse(),
        ),
        false => None,
    };

    let fuse = match (term_drain, file_drain, syslog_drain) {
        (Some(t), Some(f), Some(s)) => Async::new(Duplicate::new(Duplicate::new(t, f), s).fuse())
            .build()
            .fuse(),
        (Some(t), Some(f), None) => Async::new(Duplicate::new(t, f).fuse()).build().fuse(),
        (None, Some(f), Some(s)) => Async::new(Duplicate::new(f, s).fuse()).build().fuse(),
        (Some(t), None, Some(s)) => Async::new(Duplicate::new(t, s).fuse()).build().fuse(),
        (Some(t), None, None) => Async::new(t).build().fuse(),
        (None, Some(f), None) => Async::new(f).build().fuse(),
        (None, None, Some(s)) => Async::new(s).build().fuse(),
        (None, None, None) => Async::new(Discard).build().fuse(),
    };
    Logger::root(fuse, slog_o!())
}
