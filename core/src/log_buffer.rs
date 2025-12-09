use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub level: log::Level,
    pub target: String,
    pub message: String,
}

pub struct LogBuffer {
    entries: VecDeque<LogEntry>,
    capacity: usize,
}

impl LogBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, entry: LogEntry) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    pub fn drain(&mut self) -> Vec<LogEntry> {
        self.entries.drain(..).collect()
    }

    pub fn entries(&self) -> &VecDeque<LogEntry> {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

static LOG_BUFFER: OnceLock<Mutex<LogBuffer>> = OnceLock::new();

pub fn global_buffer() -> &'static Mutex<LogBuffer> {
    LOG_BUFFER.get_or_init(|| Mutex::new(LogBuffer::new(1024)))
}

pub struct BufferLogger;

impl log::Log for BufferLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let entry = LogEntry {
                level: record.level(),
                target: record.target().to_string(),
                message: format!("{}", record.args()),
            };
            if let Ok(mut buf) = global_buffer().lock() {
                buf.push(entry);
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: BufferLogger = BufferLogger;

pub fn init_logger(level: log::LevelFilter) -> Result<(), log::SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(level))
}

pub fn drain_logs() -> Vec<LogEntry> {
    global_buffer()
        .lock()
        .map(|mut buf| buf.drain())
        .unwrap_or_default()
}

pub fn clear_logs() {
    if let Ok(mut buf) = global_buffer().lock() {
        buf.clear();
    }
}

