use uuid::Uuid;
use web_sys::{File, Blob};

#[derive(Clone, Debug)]
pub enum FileState {
    Pending,
    Transferring,
    Done,
    Failed,
}

#[derive(Clone)]
pub struct FileTag {
    name: String,
    pub size: f64,
    uuid: Uuid,
}

impl FileTag {
    pub fn new(name: String, size: f64, uuid: Uuid) -> Self {
        Self {
            name,
            size,
            uuid,
        }
    }

    pub fn from(file: File) -> Self {
        Self {
            name: file.name(),
            size: Into::<Blob>::into(file).size(),
            uuid: Uuid::new_v4(),
        }
    }
    

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn size(&self) -> f64 {
        self.size
    }
}

impl PartialEq for FileTag {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}