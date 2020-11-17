mod ora_source;
mod types;

use std::collections::HashMap;
use std::rc::Rc;

use oracle;
use crate::utils;

pub use types::*;

use ora_source::*;

impl MetaInfo {
    pub fn new(conn: &oracle::Connection, excludes: &Vec<String>) -> oracle::OracleResult<MetaInfo> {
        let quoted_excludes: Vec<String> = excludes.iter().map(|s| format!("'{}'", s) ).collect();
        let joined_excludes = &quoted_excludes.join(",");

        let schemas = MetaInfo::load(conn, &joined_excludes)?;
        Ok( MetaInfo { schemas })
    }

    pub fn load(conn: &oracle::Connection, excludes: &str)-> oracle::OracleResult<HashMap<Rc<String>,SchemaInfo>> {
        // tables and columns queries/iterators are sorted by owner, table_name and synchronized
        use itertools::Itertools;

        let tables_iterator = fetch_tables(conn, excludes)?;
        let columns_iterator = fetch_columns(conn, excludes)?;

        // group tables and columns iterators by schema name
        // TODO: log errors in ecah result (see: filter_map)

        let grouped_tables = tables_iterator
            .filter_map(|r|r.ok())
            .group_by(|t| t.owner.clone() );

        let grouped_columns = columns_iterator
            .filter_map(|r|r.ok())
            .group_by(|t| t.owner.clone() );

        // join tables and columns grouped iterators
        let joined = grouped_tables.into_iter().zip(grouped_columns.into_iter()).map(|entry| {
            let tables = entry.0;
            let columns = entry.1;

            // name of schema in tables and columns iterators must same
            assert_eq!(tables.0, columns.0);
            (tables.0, tables.1, columns.1)
        });

        let mut result = HashMap::with_capacity(100);

        for (schema, tables, columns) in joined {
            // group columns iterator by table name
            let grouped_columns = columns.group_by(|t|t.table_name.clone());
            // join tables and columns iterators
            let joined = tables.zip(grouped_columns.into_iter()).map(|entry| {
                let table = entry.0;
                let columns = entry.1;

                // name of table in tables and columns iterator must same
                assert_eq!(table.table_name, columns.0);
                (table, columns.1)
            });

            let mut tables = HashMap::with_capacity(200);

            for (table,columns) in joined {
                // construct table info and push it to tables map
                let name = Rc::new(table.table_name);
                let num_rows = table.num_rows;

                let is_view = table.table_type == "VIEW";
                let temporary = table.temporary == "Y";

                // construct column info and collect it to vector of columns
                let columns = columns.map(|c|c.into()).collect();

                let table = TableInfo { name: name.clone(), is_view, temporary, num_rows, columns, primary_key: None, indexes: Vec::new() };
                tables.insert(name, table);
            }

            let name = Rc::new(schema);
            let schema = SchemaInfo { name: name.clone(), tables};

            result.insert(name, schema);
        };

        Ok(result)
    }

}
