use std::backtrace::Backtrace;
use std::env;
use std::io::Write;
use std::net::TcpListener;
use std::ops::Deref;
use std::time::Instant;
use std::{panic, thread};

use log::{error, info, set_boxed_logger, set_max_level, LevelFilter, Log, Metadata, Record};
use mesura::get_metrics;

struct BasicLogger {
    start: Instant,
}

impl BasicLogger {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Log for BasicLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let timestamp = Instant::now().duration_since(self.start).as_secs_f32();
        println!(
            "{:.4} {} [{}] {}",
            timestamp,
            record.level(),
            record.module_path().unwrap_or("unknown"),
            record.args()
        )
    }

    fn flush(&self) {}
}

pub fn setup_basic_logging(level: LevelFilter) {
    set_boxed_logger(Box::new(BasicLogger::new())).expect("basic logger must be init");
    set_max_level(level);

    panic::set_hook(Box::new(|info| {
        let (file, line) = info
            .location()
            .map(|location| (location.file(), location.line()))
            .unwrap_or(("<unknown>", 0));

        let current = thread::current();
        let name = current.name().unwrap_or("<unnamed>");

        let reason = info
            .payload()
            .downcast_ref::<String>()
            .map(String::deref)
            .unwrap_or_else(|| {
                info.payload()
                    .downcast_ref::<&str>()
                    .map(|string| *string)
                    .unwrap_or("<undescribed>")
            });

        error!("thread {name} panic! at {}:{}: {}", file, line, reason);
        info!("{}", Backtrace::force_capture());
    }));

    info!("Starts logging");
}

pub fn setup_basic_monitoring() {
    let host = env::var("MONITORING_PORT")
        .map(|port| format!("0.0.0.0:{port}"))
        .ok();
    thread::Builder::new()
        .name("monitoring".into())
        .spawn(|| serve_prometheus_metrics(host))
        .expect("monitoring thread must be spawned");
}

fn serve_prometheus_metrics(host: Option<String>) {
    match host {
        None => {
            info!("Disables monitoring, port not specified via MONITORING_PORT env variable");
        }
        Some(host) => {
            info!("Starts monitoring endpoint at {host}");
            let listener = TcpListener::bind(host).expect("listener must be bound");
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let status = "HTTP/1.1 200 OK";
                let contents = {
                    // NOTE: minimize lock in scope
                    let registry = get_metrics()
                        .read()
                        .expect("registry must be valid to read");
                    registry.encode_prometheus_report()
                };
                let len = contents.len();
                let response = format!("{status}\r\nContent-Length: {len}\r\n\r\n{contents}");
                stream
                    .write_all(response.as_bytes())
                    .expect("metrics response must be written");
            }
        }
    }
}
