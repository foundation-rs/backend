// #[macro_use]
use lazy_static::lazy_static;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
use crate::oci;

/// Oracle environment
pub struct Environment {
    pub(crate) envhp: *mut oci::OCIEnv,
    pub(crate) errhp: *mut oci::OCIError
}

type EnvironmentResult = Result<Environment, oci::OracleError>;

// for multithreading and lazy_static
unsafe impl Sync for Environment {}
unsafe impl Send for Environment {}

lazy_static! {
  static ref ORACLE_ENV: EnvironmentResult = Environment::new();
}

impl Environment {

    /// Create new environment
    fn new() -> Result<Environment, oci::OracleError> {
        let envhp = oci::env_create()?;
        // create error handle
        let errhp = oci::handle_alloc(envhp, oci::OCI_HTYPE_ERROR)? as *mut oci::OCIError;
        Ok(Environment{ envhp, errhp })
    }

    pub fn get() -> Result<&'static Environment, oci::OracleError> {
        match *ORACLE_ENV {
            Ok(ref env) => Ok(env),
            Err(ref err) => Err(err.to_owned())
        }
    }

}

impl Drop for Environment {
    fn drop(&mut self) {
        oci::handle_free(self.errhp as *mut oci::c_void, oci::OCI_HTYPE_ERROR);
        oci::handle_free(self.envhp as *mut oci::c_void, oci::OCI_HTYPE_ENV);
        oci::terminate();
    }
}
