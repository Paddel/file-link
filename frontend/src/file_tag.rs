use uuid::Uuid;
use web_sys::{File, Blob};

#[derive(Clone, Debug, PartialEq)]
pub enum FileState {
    Pending,
    Transferring,
    Done,
    Queued,
}

#[derive(Clone)]
pub struct FileTag {
    name: String,
    pub size: f64,
    pub uuid: Uuid,
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

pub fn convert_bytes_to_readable_format(bytes: u64) -> String {
    const KILO: u64 = 1024;
    const MEGA: u64 = KILO * 1024;
    const GIGA: u64 = MEGA * 1024;
    const TERA: u64 = GIGA * 1024;

    if bytes < KILO {
        return format!("{} Bytes", bytes);
    } else if bytes < MEGA {
        return format!("{:.2} KB", (bytes as f64) / (KILO as f64));
    } else if bytes < GIGA {
        return format!("{:.2} MB", (bytes as f64) / (MEGA as f64));
    } else if bytes < TERA {
        return format!("{:.2} GB", (bytes as f64) / (GIGA as f64));
    } else {
        return format!("{:.2} TB", (bytes as f64) / (TERA as f64));
    }
}
