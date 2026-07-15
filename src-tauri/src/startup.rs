use std::sync::Mutex;

pub struct StartupStatus(Mutex<Option<String>>);

impl StartupStatus {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }

    pub fn succeed(&self) {
        *self.0.lock().expect("startup lock poisoned") = None;
    }

    pub fn fail(&self, msg: &str) {
        *self.0.lock().expect("startup lock poisoned") = Some(msg.to_string());
    }

    pub fn get(&self) -> Option<String> {
        self.0.lock().expect("startup lock poisoned").clone()
    }
}
