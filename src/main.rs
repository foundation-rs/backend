#![feature(min_const_generics)]
#![feature(option_insert)]

mod config;
mod utils;
mod metainfo;

use oracle;

fn main() -> Result<(), String> {
    let start = chrono::offset::Local::now();

    let ref conf = config::load("config.xml")?;
    let ref cc = conf.connection;

    let conn = oracle::connect(&cc.url, &cc.user, &cc.pw)
        .map_err(|err| format!("Can not connect to Oracle: {}", err))?;

    let mi = metainfo::MetaInfo::new(&conn, &conf.excludes.schemes)
                .map_err(|err| format!("Can not read metainfo about oracle tables: {}", err))?;

    let mut v: Vec<_> = mi.schemas.iter().collect();
    v.sort_by(|x,y| x.0.as_ref().cmp(&y.0.as_ref()));

    let mut schemas_count = 0;
    let mut tables_count = 0;
    let mut columns_count = 0;

    let mut max_columns_count = 0;

    for (key,schema) in v.iter() {
        // println!();
        // println!("[{}]", key.as_ref());

        let mut v: Vec<_> = schema.tables.iter().collect();
        v.sort_by(|x,y| x.0.as_ref().cmp(&y.0.as_ref()));
    
        for (key,table) in v {
            // println!("{}; rows: {}", key.as_ref(), table.num_rows);
            tables_count += 1;
            columns_count += table.columns.len();

            use std::cmp;
            max_columns_count = cmp::max(max_columns_count, table.columns.len());
        }
        schemas_count += 1;
    }

    println!();
    println!("TOTAL:   {} schemas with {} tables & views and {} columns", schemas_count,  tables_count, columns_count);

    println!("         maximum count of columns in table: {}", max_columns_count);

    let end = chrono::offset::Local::now();
    let duration = end - start;

    let seconds = duration.num_seconds();
    let milliseconds = duration.num_milliseconds() - seconds * 1000;
    println!();
    println!("ELAPSED: {} seconds, {} milliseconds", seconds, milliseconds);

    Ok(())
}
