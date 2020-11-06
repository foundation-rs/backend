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
        println!("t {}.{}", t.owner, t.table_name);
    }
    println!("total tables: {}", tables.len());

    Ok(())
}

#[derive(Query)]
pub struct OraTable {
    owner: String,
    table_name: String
}

pub fn load(conn: &oracle::Connection, excludes: &Vec<String>) -> Result<Vec<OraTable>,oracle::OracleError> {
    use std::ops::Add;

    let quoted_excludes: Vec<String> = excludes.iter().map(|s| format!("'{}'", s) ).collect();
    let sql = format!(
        "SELECT OWNER, TABLE_NAME FROM SYS.ALL_TABLES WHERE OWNER NOT IN ( {} ) ORDER BY OWNER, TABLE_NAME",
            &quoted_excludes.join(","));

    let mut result = Vec::with_capacity(8000);
    let mut query = conn.query::<OraTable>(&sql)?;

    for v in query.fetch_iter()? {
        if let Ok(v) = v {
            result.push(v);
        };
    }

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
            let p0 = projecton.get_unchecked_mut(0);
            &self.id.project_value(p0);
        }

        unsafe {
            let p1 = projecton.get_unchecked_mut(1);
            &self.name.project_value(p1);
        }
    }

}
