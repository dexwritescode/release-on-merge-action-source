use std::fs::write;

pub struct Writer {
    path: String
}

impl Writer {
    pub fn new(path: String) -> Writer {
        Writer {
            path,
        }
    }

    pub fn write_value(&self, value: &str) {
        write(&self.path, value).unwrap();
    }
    
    pub fn write(&self, key: &str, value: &str) {
        write(&self.path, format!("{key}={value}")).unwrap();
    }
}
