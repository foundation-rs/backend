mod environment;
mod oci;
mod connection;
mod types;
mod values;
mod dates;
mod statement;
mod singulars;

pub use oci::{OracleError, OracleResult};
pub use connection::{Connection, connect};

pub use dates::{SqlDate, SqlDateTime};

pub use types::{
    DescriptorsProvider,
    TypeDescriptor,
    TypeDescriptorProducer
};

pub use statement::{
    Statement,
    Query,
    ResultsProvider,
    ResultSet,
    ParamsProvider,
    ParamsProjection,
};

pub use statement::params::{
    Identifier,
    Member,
    ValueProjector
};
