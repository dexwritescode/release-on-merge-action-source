use std::fs::write;

pub struct Writer {
    path: String,
}

impl Writer {
    pub fn new(path: String) -> Writer {
        Writer { path }
    }

    pub fn write(&self, key: &str, value: &str) {
        write(&self.path, format!("{key}={value}")).unwrap();
    }
}
