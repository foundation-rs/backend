mod config;

// TODO: use xml for config
// SEE: https://github.com/tafia/quick-xml

use oracle;
use oracle_derive::Query;
use oracle::ValueProjector;

fn main() -> Result<(), &'static str> {
    let ref conf = config::load("config.json")?;

    let ref cc = conf.connection;

    let conn = oracle::connect(&cc.url, &cc.user, &cc.pw)
        .map_err(|err|"Can not connect to Oracle")?;

    let tables = load(&conn, &conf.excludes)
        .map_err(|err| "Can not read metainfo abaut oracle tables")?;
    for t in &tables {
        println!("t {}.{}; rows: {}", t.owner, t.table_name, t.num_rows);
    }
    println!("total tables: {}", tables.len());

    Ok(())
}

#[derive(Query)]
pub struct OraTable {
    owner: String,
    table_name: String,
    num_rows: i32
}

#[derive(Query)]
pub struct OraTableColumn {
    column_id: i16,
    owner: String,
    table_name: String,
    column_name: String,
    data_type: String,
    data_length: i16,
    nullable: String
}

// TODO: convert String to &'a str
// TODO: proper lifetimes
pub struct OraTableColumnParams (String, String);

pub fn load(conn: &oracle::Connection, excludes: &Vec<String>) -> Result<Vec<OraTable>,oracle::OracleError> {
    use std::ops::Add;

    let quoted_excludes: Vec<String> = excludes.iter().map(|s| format!("'{}'", s) ).collect();
    let sql = format!(
        "SELECT OWNER, TABLE_NAME, NUM_ROWS FROM SYS.ALL_TABLES WHERE OWNER NOT IN ( {} ) ORDER BY OWNER, TABLE_NAME",
            &quoted_excludes.join(","));

    let sql_cols =
        "SELECT COLUMN_ID, OWNER, TABLE_NAME, COLUMN_NAME, DATA_TYPE, DATA_LENGTH, NULLABLE \
        FROM SYS.ALL_TAB_COLUMNS WHERE OWNER = :1 AND TABLE_NAME = :2";

    let mut result = Vec::with_capacity(8000);

    let mut query = conn
        .prepare(&sql)?
        .query_many::<OraTable>(1000)?;

    let mut colmns_query = conn
        .prepare(&sql_cols)?
        .params::<OraTableColumnParams>()?
        .query_many::<OraTableColumn>(100)?;

    let mut columns_cnt = 0;

    for v in query.fetch_iter()? {
        if let Ok(v) = v {
            {
                let params = OraTableColumnParams (v.owner.clone(), v.table_name.clone());
                let columns = colmns_query.fetch_list(&params)?;
                for c in columns {
                    // println!("   c {} {}", c.column_name, c.data_type);
                    columns_cnt +=1;
                }
            }
            result.push(v);
        };
    }

    println!("total columns: {}", columns_cnt);

    Ok(result)
}

#[derive(Query)]
pub struct TestingTuple (i32, String, String);

impl oracle::ParamsProvider for TestingTuple {
    fn members() -> Vec<oracle::Member> {
        use oracle::TypeDescriptorProducer;
        vec![
            oracle::Member::new(i32::produce(), oracle::Identifier::Unnamed),
            oracle::Member::new(String::produce(), oracle::Identifier::Unnamed),
            oracle::Member::new(String::produce(), oracle::Identifier::Unnamed),
            ]
    }

    fn project_values(&self, projecton: &mut oracle::ParamsProjection) -> () {
        unsafe {
            let p = projecton.get_unchecked_mut(0);
            &self.0.project_value(p);
        }

        unsafe {
            let p = projecton.get_unchecked_mut(1);
            &self.1.project_value(p);
        }

        unsafe {
            let p = projecton.get_unchecked_mut(2);
            &self.2.project_value(p);
        }
    }

}

impl oracle::ParamsProvider for OraTableColumnParams {
    fn members() -> Vec<oracle::Member> {
        use oracle::TypeDescriptorProducer;
        vec![
            oracle::Member::new(String::produce(), oracle::Identifier::Unnamed),
            oracle::Member::new(String::produce(), oracle::Identifier::Unnamed),
        ]
    }

    fn project_values(&self, projecton: &mut oracle::ParamsProjection) -> () {
        unsafe {
            let p = projecton.get_unchecked_mut(0);
            &self.0.project_value(p);
        }

        unsafe {
            let p = projecton.get_unchecked_mut(1);
            &self.1.project_value(p);
        }
    }

}

pub struct TestingTuple2<'a> {
    id: i32,
    name: &'a str
}

impl <'a> oracle::ParamsProvider for TestingTuple2<'a> {
    fn members() -> Vec<oracle::Member> {
        use oracle::TypeDescriptorProducer;
        vec![
            oracle::Member::new(i32::produce(), oracle::Identifier::Named("id")),
            oracle::Member::new(String::produce(), oracle::Identifier::Named("name")),
        ]
    }

    fn project_values(&self, projecton: &mut oracle::ParamsProjection) -> () {
        unsafe {
            let p = projecton.get_unchecked_mut(0);
            &self.id.project_value(p);
        }

        unsafe {
            let p = projecton.get_unchecked_mut(1);
            &self.name.project_value(p);
        }
    }

}
