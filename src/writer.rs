use std::fs::write;

pub struct Writer {
    path: String,
}

impl Writer {
    pub fn new(path: &str) -> Writer {
        Writer {
            path: path.to_owned(),
        }
    }

    pub fn write(&self, key: &str, value: &str) {
        write(&self.path, format!("{key}={value}")).unwrap();
    }
}
