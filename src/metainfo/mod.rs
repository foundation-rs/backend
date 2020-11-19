use std::collections::HashMap;
use std::rc::Rc;
use itertools::Itertools;

use oracle;
use crate::datasource;

mod ora_source;
mod types;

pub use types::*;
use ora_source::*;
use crate::config::Excludes;

impl MetaInfo {
    pub fn load(excludes: &Excludes) -> Result<MetaInfo, String> {
        std::thread::sleep(std::time::Duration::from_millis(10));
        println!();
        println!("READING METAINFO FROM ORACLE...");

        let start = chrono::offset::Local::now();

        let conn = datasource::get_connection()
            .map_err(|err|format!("Can not connect to oracle: {}", err))?;

        let schemas = MetaInfo::load_internal(&conn, &excludes.schemas)
            .map_err(|err| format!("Can not read metainfo about oracle tables: {}", err))?;

        let mut schemas_count = 0;
        let mut tables_count = 0;
        let mut columns_count = 0;
        let mut pks_count = 0;
        let mut indexes_count = 0;

        for (_key,schema) in schemas.iter() {
            for (_key,table) in schema.tables.iter() {
                tables_count += 1;
                columns_count += table.columns.len();
                indexes_count += table.indexes.len();

                if table.primary_key.is_some() {
                    pks_count += 1;
                }
            }
            schemas_count += 1;
        }

        println!();
        println!("TOTAL:   {} schemas with {} tables & views and {} columns", schemas_count,  tables_count, columns_count);
        println!("         {} tables with primary keys", pks_count);
        println!("         {} indexes found", indexes_count);

        let end = chrono::offset::Local::now();
        let duration = end - start;

        let seconds = duration.num_seconds();
        let milliseconds = duration.num_milliseconds() - seconds * 1000;

        println!();
        println!("ELAPSED: {} seconds, {} milliseconds", seconds, milliseconds);
        println!();

        Ok( MetaInfo { schemas })
    }

    fn load_internal(conn: &oracle::Connection, excludes: &Vec<String>) -> oracle::OracleResult<HashMap<Rc<String>,SchemaInfo>> {
        let quoted_excludes: Vec<String> = excludes.iter().map(|s| format!("'{}'", s) ).collect();
        let joined_excludes = &quoted_excludes.join(",");

        let mut schemas = MetaInfo::load_tables(&conn, &joined_excludes)?;
        MetaInfo::load_primary_keys(&conn, &joined_excludes, &mut schemas)?;
        MetaInfo::load_indexes(&conn, &joined_excludes, &mut schemas)?;

        Ok(schemas)
    }

    fn load_tables(conn: &oracle::Connection, excludes: &str) -> oracle::OracleResult<HashMap<Rc<String>,SchemaInfo>> {
        // tables and columns queries/iterators are sorted by owner, table_name and synchronized
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
        let joined = grouped_tables.into_iter().zip(grouped_columns.into_iter()).map(|(tables,columns)| {
            // name of schema in tables and columns iterators must same
            assert_eq!(tables.0, columns.0);
            (tables.0, tables.1, columns.1)
        });

        let mut result = HashMap::with_capacity(100);

        for (schema, tables, columns) in joined {
            // group columns iterator by table name
            let grouped_columns = columns.group_by(|t|t.table_name.clone());
            // join tables and columns iterators
            let joined = tables.zip(grouped_columns.into_iter()).map(|(table,columns)| {
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

    fn load_primary_keys(conn: &oracle::Connection, excludes: &str, schemas: &mut HashMap<Rc<String>,SchemaInfo>) -> oracle::OracleResult<()> {
        let pk_iterator = fetch_primary_keys(conn, excludes)?;

        // group primary keys by schema
        let grouped_keys = pk_iterator
            .filter_map(|r|r.ok())
            .group_by(|t| t.owner.clone() );

        for (schema, keys) in grouped_keys.into_iter() {
            let schema = schemas.get_mut(&schema);

            if let Some(schema) = schema {
                // group keys by table name and constraint name
                let grouped_keys = keys
                    .group_by(|t| (t.table_name.clone(),t.constraint_name.clone()) );

                for ((table_name, name), key_columns) in grouped_keys.into_iter() {
                    let table_info = schema.tables.get_mut(&table_name);
                    if let Some(table_info) = table_info {
                        let columns = key_columns.map(|c|c.column_name).collect();
                        table_info.primary_key = Some(PrimaryKey { name, columns});
                    } // table info found
                }
            } // schema found
        };

        Ok(())
    }

    fn load_indexes(conn: &oracle::Connection, excludes: &str, schemas: &mut HashMap<Rc<String>,SchemaInfo>) -> oracle::OracleResult<()> {
        let idx_iterator = fetch_indexes(conn, excludes)?;

        // group indexes by schema
        let grouped_indexes = idx_iterator
            .filter_map(|r|r.ok())
            .group_by(|t| t.owner.clone() );

        for (schema, indexes) in grouped_indexes.into_iter() {
            let schema = schemas.get_mut(&schema);

            if let Some(schema) = schema {
                // group indexes by table name and index name
                let grouped_indexes = indexes
                    .group_by(|t| t.table_name.clone() );

                for (table_name, indexes) in grouped_indexes.into_iter() {
                    let table_info = schema.tables.get_mut(&table_name);
                    if let Some(table_info) = table_info {
                        let indexes = indexes.group_by(|t|(t.index_name.clone(), t.uniqueness.clone()));

                        for ((index_name, uniqueness), columns) in indexes.into_iter() {
                            let columns = columns.map(|c| IndexColumn{name: c.column_name, desc: c.descend != "ACC"}).collect();
                            let index = TableIndex {name: index_name, unique: uniqueness == "UNIQUE", columns};
                            table_info.indexes.push(index);
                        }
                    } // table info found
                }
            } // schema found
        };

        Ok(())
    }

}
