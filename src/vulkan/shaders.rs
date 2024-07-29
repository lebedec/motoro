use std::fs;
use std::time::SystemTime;

pub struct Shader {
    path: String,
    version: SystemTime,
}

impl Shader {
    pub fn new(path: &str) -> Shader {
        Self {
            version: Self::modified(path),
            path: path.to_string(),
        }
    }

    pub fn renew(&self) -> Shader {
        Self::new(&self.path)
    }

    pub fn modified(path: &str) -> SystemTime {
        let metadata = fs::metadata(path).expect("metadata must be available");
        metadata
            .modified()
            .expect("modified time must be available")
    }

    pub fn changed(&self) -> bool {
        self.version != Self::modified(&self.path)
    }

    pub fn read(&mut self) -> Vec<u8> {
        fs::read(&self.path).expect("file must be read")
    }
}
