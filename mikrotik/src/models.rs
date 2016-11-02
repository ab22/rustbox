#[derive(Clone)]
pub struct IPAddress {
    pub id: String,
    pub address: String,
    pub list: String,
}

impl IPAddress {
    pub fn new() -> IPAddress {
        IPAddress {
            id: String::new(),
            address: String::new(),
            list: String::new(),
        }
    }
}

#[derive(Clone)]
pub struct Client {
    pub id: String,
    pub name: String,
    pub target: String,
    pub max_limit: String,
    pub burst_limit: String,
    pub burst_threshold: String,
    pub burst_time: String,
}


impl Client {
    pub fn new() -> Client {
        Client {
            id: String::new(),
            name: String::new(),
            target: String::new(),
            max_limit: String::new(),
            burst_limit: String::new(),
            burst_threshold: String::new(),
            burst_time: String::new(),
        }
    }
}
