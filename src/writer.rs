use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::LineWriter;

pub struct Writer {
    file: std::io::LineWriter<std::fs::File>,
}

impl Writer {
    pub fn new(path: &str) -> Writer {
        Writer {
            file: LineWriter::new(OpenOptions::new().write(true).open(path).unwrap()),
        }
    }

    pub fn write(&mut self, key: &str, value: &str) {
        self.file
            .write_all(format!("{key}={value}\n").as_bytes())
            .unwrap();
    }
}
