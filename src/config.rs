use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
pub struct Connection {
    pub url: String,
    pub user: String,
    pub pw: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub connection: Connection,
    pub excludes: Vec<String>,
}

pub fn load(filename: &str) -> Result<Config, &'static str> {
    let mut file = File::open(filename).map_err(|err| "Can not open config file")?;
    let mut data = String::new();
    file.read_to_string(&mut data).map_err(|err| "Can not read config file")?;

    serde_json::from_str(&data).map_err(|err| "Can not parse config file")
}