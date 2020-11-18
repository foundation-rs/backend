use std::fs::File;
use std::io::Read;

use serde::{Deserialize};
use quick_xml::de::from_str;

// https://github.com/tafia/quick-xml

#[derive(Deserialize, Debug, PartialEq)]
pub struct Config {
    pub connection: Connection,
    pub excludes: Excludes,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Connection {
    pub url: String,
    pub user: String,
    pub pw: String
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Excludes {
    #[serde(rename = "schema", default)]
    pub schemas: Vec<String>,
}

pub fn load(filename: &str) -> Result<Config, String> {
    let mut file = File::open(filename).map_err(|err| format!("Can not open config file: {}", err))?;
    let mut data = String::new();
    file.read_to_string(&mut data).map_err(|err| format!("Can not read config file: {}", err))?;

    from_str(&data).map_err(|err| format!("Can not parse config file: {}", err))
}