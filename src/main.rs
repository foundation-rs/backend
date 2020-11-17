#![feature(min_const_generics)]
#![feature(option_insert)]

use std::env;

mod config;
mod utils;
mod metainfo;

use oracle;

fn main() -> Result<(), String> {
    let start = chrono::offset::Local::now();

    let ref conf = config::load("config.xml")?;
    let ref cc = conf.connection;

    let url = &cc.url;
    let user = &cc.user;
    let mut pw = cc.pw.clone();

    if (&cc.pw).starts_with("env:") {
        let key = &cc.pw[4..];
        pw = env::var(key).unwrap_or(pw);
    };

    let conn = oracle::connect(url, user, &pw)
        .map_err(|err| format!("Can not connect to Oracle: {}", err))?;

    let mi = metainfo::MetaInfo::new(&conn, &conf.excludes.schemes)
                .map_err(|err| format!("Can not read metainfo about oracle tables: {}", err))?;

    let mut v: Vec<_> = mi.schemas.iter().collect();
    v.sort_by(|x,y| x.0.as_ref().cmp(&y.0.as_ref()));

    let mut schemas_count = 0;
    let mut tables_count = 0;
    let mut columns_count = 0;
    let mut pks_count = 0;
    let mut indexes_count = 0;

    for (key,schema) in v.iter() {
        // println!();
        // println!("[{}]", key.as_ref());

        let mut v: Vec<_> = schema.tables.iter().collect();
        v.sort_by(|x,y| x.0.as_ref().cmp(&y.0.as_ref()));
    
        for (key,table) in v {
            // println!("{}; rows: {}", key.as_ref(), table.num_rows);
            tables_count += 1;
            columns_count += table.columns.len();

            if table.primary_key.is_some() {
                pks_count += 1;
            }

            indexes_count += table.indexes.len();
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

    Ok(())
}
