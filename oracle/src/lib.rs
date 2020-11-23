mod environment;
mod oci;
mod connection;
mod types;
mod values;
mod sql_types;
mod statement;
mod singulars;

pub use oci::{OracleError, OracleResult};
pub use connection::{Connection, SessionPool, create_pool};

pub use sql_types::{SqlDate, SqlDateTime};

pub use types::{
    DescriptorsProvider,
    TypeDescriptor,
    TypeDescriptorProducer
};

pub use statement::{
    Statement,
    Query,
    QueryIterator,
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
