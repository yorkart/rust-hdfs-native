

pub struct Common {
    pub nn_hostport: String,
    pub effective_user: String,
    pub nat: Vec<(String, String)>
}

impl Default for Common {
    fn default() -> Self {
        Common {
            nn_hostport: "local".to_owned(),
            effective_user: "cloudera".to_owned(),
            nat: vec![]
        }
    }
}

#[derive(Debug)]
pub struct GetListing {
    pub src: Vec<String>,
    pub need_location: bool
}

pub struct Get {
    pub src: Vec<String>,
    pub tgt_dir: Option<String>
}

