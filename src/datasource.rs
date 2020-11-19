use std::env;
use std::sync::RwLock;

// #[macro_use]
use lazy_static::lazy_static;

use crate::config::ConnectionConfig;
use oracle;

pub struct Datasource {
    url: String,
    user: String,
    pw: String
}

type DatasourceHandler = RwLock<Option<Datasource>>;

lazy_static! {
  static ref DATASOURCE: DatasourceHandler = RwLock::new(None);
}

impl Datasource {
    fn new(config: &ConnectionConfig) -> Datasource {
        let url = &config.url;
        let user = &config.user;
        let mut pw = config.pw.clone();

        if (&config.pw).starts_with("env:") {
            let key = &config.pw[4..];
            pw = env::var(key).unwrap_or(pw);
        };

        Datasource { url: url.to_string(), user: user.to_string(), pw}
    }

}

pub fn create(config: &ConnectionConfig) -> Result<(), String> {
    let mut ds = (*DATASOURCE).write()
        .map_err(|_err| format!("Can not get lock for datasource creation"))?;

    if let None = *ds {
        *ds = Some(Datasource::new(config));
    };

    Ok(())
}

pub fn get_connection() -> oracle::OracleResult<oracle::Connection> {
    let ds = (*DATASOURCE).read().unwrap();
    let cc = ds.as_ref().unwrap();
    oracle::connect(&cc.url, &cc.user, &cc.pw)
}
