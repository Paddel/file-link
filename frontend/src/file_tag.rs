use uuid::Uuid;
use web_sys::{File, Blob};

#[derive(Clone)]
pub struct FileTag {
    file: File,
    uuid: Uuid,
}

impl FileTag {
    pub fn new(file: File) -> Self {
        Self {
            file,
            uuid: Uuid::new_v4(),
        }
    }

    pub fn blob(self) -> Blob {
        self.file.into()
    }

    pub fn name(&self) -> String {
        self.file.name()
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }
}

impl PartialEq for FileTag {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}