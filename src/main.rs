mod config;
mod metainfo;

use oracle;

fn main() -> Result<(), String> {
    let start = chrono::offset::Local::now();

    let ref conf = config::load("config.xml")?;
    let ref cc = conf.connection;

    let conn = oracle::connect(&cc.url, &cc.user, &cc.pw)
        .map_err(|err| format!("Can not connect to Oracle: {}", err))?;

    let mi =  metainfo::MetaInfo::new(&conn, &conf.excludes.schemes)
                .map_err(|err| format!("Can not read metainfo about oracle tables: {}", err))?;

    println!("TOTAL: {} schemas with {} tables & views and {} columns",
        &mi.schemas.len(), 
        &mi.schemas.values().map(|s|s.tables.len()).sum::<usize>(),
        &mi.schemas.values().map(|s|s.tables.values().map(|t|t.columns.len()).sum::<usize>()).sum::<usize>()
    );

    let end = chrono::offset::Local::now();
    let duration = end - start;

    let seconds = duration.num_seconds();
    let milliseconds = duration.num_milliseconds() - seconds * 1000;
    println!("ELAPSED: {} seconds, {} milliseconds", seconds, milliseconds);

    Ok(())
}
