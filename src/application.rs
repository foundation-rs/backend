use std::sync::Mutex;
use std::io::{Error, ErrorKind, Result};

use crate::config::Config;
use crate::metainfo::{self,MetaInfo};

// This struct represents state
pub struct ApplicationState {
    metainfo: Mutex<MetaInfo>
}

impl ApplicationState {
    pub fn load(conf: &Config) -> Result<ApplicationState> {
        let metainfo = metainfo::MetaInfo::load(&conf.excludes)
            .map_err(|e|Error::new(ErrorKind::Other, e))?;
        let metainfo = Mutex::new(metainfo);
        Ok( ApplicationState{metainfo} )
    }
}

