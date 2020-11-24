use std::env;
use std::sync::RwLock;

use lazy_static::lazy_static;

use crate::config::ConnectionConfig;
use oracle;

pub struct Datasource {
    pool: oracle::SessionPool,
}

type DatasourceHandler = RwLock<Option<Datasource>>;

lazy_static! {
  static ref DATASOURCE: DatasourceHandler = RwLock::new(None);
}

impl Datasource {
    fn new(config: &ConnectionConfig) -> oracle::OracleResult<Datasource> {
        let url = &config.url;
        let user = &config.user;
        let mut pw = config.pw.clone();

        if (&config.pw).starts_with("env:") {
            let key = &config.pw[4..];
            pw = env::var(key).unwrap_or(pw);
        };

        let pool = oracle::create_pool(url, user, &pw)?;
        Ok(Datasource{pool})
    }
}

pub fn create(config: &ConnectionConfig) -> Result<(), String> {
    let mut ds = (*DATASOURCE).write()
        .map_err(|_err| format!("Can not get lock for datasource creation"))?;

    if let None = *ds {
        let datasource = Datasource::new(config)
            .map_err(|err| format!("Can not create connection pool: {}", err))?;
        *ds = Some(datasource);
    };

    Ok(())
}

pub fn get_connection() -> oracle::OracleResult<oracle::Connection> {
    let ds = (*DATASOURCE).read().unwrap();
    let cc = ds.as_ref().unwrap();
    // oracle::connect(&cc.url, &cc.user, &cc.pw)
    cc.pool.connect()
}
