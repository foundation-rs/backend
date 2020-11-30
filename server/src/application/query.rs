use oracle::{self, ValueProjector};
use crate::{metainfo as mi, datasource};

pub struct DynamicQuery {
    sql:           String,
    column_names:  Vec<String>,
    columns:       Vec<ColTypeInfo>,
    param_columns: Vec<ColTypeInfo>,
    parsed_params: Vec<ParsedParameter>
}

struct DynamicResultsProvider {
    column_names: Vec<String>,
    columns:      Vec<ColTypeInfo>
}

struct DynamicParamsProvider {
    columns: Vec<ColTypeInfo>
}

enum ParsedParameter {
    Int16 (i16), Int32(i32), Int64(i64), Varchar(String)
}

struct ColTypeInfo {
    pub col_type:      oracle::SqlType,
    pub oci_data_type: oracle::TypeDescriptor,
}

impl ColTypeInfo {
    fn new(info: &mi::ColumnInfo) -> ColTypeInfo {
        ColTypeInfo { col_type: info.col_type, oci_data_type: info.oci_data_type }
    }
}

impl DynamicQuery {
    pub fn create_from_pk(schema_name: &str, table_info: &mi::TableInfo, parameter: String) -> Result<DynamicQuery, &'static str> {
        match &table_info.primary_key {
            None => Err("Primary key not exists"),
            Some(pk) => {
                let pk_indices = &pk.column_indices;
                if pk_indices.len() > 1 {
                    return Err("Primary key must have only ONE column")
                }
                let pk_column_index = unsafe { pk_indices.get_unchecked(0) };
                let pk_column = unsafe { table_info.columns.get_unchecked(*pk_column_index) };

                let table_name = table_info.name.as_str();
                let columns: Vec<ColTypeInfo> = table_info.columns.iter().map(ColTypeInfo::new).collect();
                let column_names = table_info.columns.iter().map(|c|c.name.as_str()).collect();

                let param_column_names = vec![pk_column.name.as_str()];
                let pk_column = ColTypeInfo::new( pk_column );

                let sql = DynamicQuery::generate_sql(schema_name, table_name, &column_names, &param_column_names);
                let column_names = column_names.iter().map(|name|name.to_string()).collect();

                ParsedParameter::parse(pk_column.col_type, parameter)
                    .map(|parsed_parameter|DynamicQuery{sql, columns, column_names, param_columns: vec![pk_column], parsed_params: vec![parsed_parameter]})
            }
        }
    }

    /*
    pub fn create_from_params(schema_name: &'a str, table_info: &'a mi::TableInfo, parameters: Vec<(String,String)>) -> Result<DynamicQuery<'a>, String> {
        let columns: Vec<(&mi::ColumnInfo, ParsedParameter)> = parameters.iter().map(|(name,value)| {
            let column = table_info.columns.iter().find(|c|c.name == name);

            if let Some(column) = column {
                let parsed = ParsedParameter::parse(column.col_type, value.to_string());
                if let Err(err) = parsed {
                    Err(format!("Can not parse parameter value {} for column {}: {}", value, column.name, err));
                } else {
                    Ok((column, parsed))
                }
            } else {
                Err(format!("Not found column {}", name))
            }
        }).filter_map(|c|c.ok()).collect();

        let mut param_columns = Vec::with_capacity(columns.len());
        let mut parsed_params = Vec::with_capacity(columns.len());

        for (c,p) in columns {
            param_columns.push(c);
            parsed_params.push(p);
        }

        Ok( DynamicQuery { schema_name, table_info, param_columns, parsed_params } )
    }
    */

    pub fn generate_sql(schema_name: &str, table_name: &str, column_names: &Vec<&str>, param_column_names: &Vec<&str>) -> String {
        let joined_result_columns = column_names.join(",");

        let enumerated_param_columns: Vec<String> =
            param_column_names.iter().enumerate().map(|(idx,name)|format!("{} = {}", name, idx+1)).collect();
        let joined_param_columns = enumerated_param_columns.join(" AND ");

        format!("SELECT {} FROM {}.{} WHERE {}", joined_result_columns, schema_name,  table_name, joined_param_columns)
    }

    /// execute a query and generate JSON result
    pub fn execute(self) -> Result<String,String> {
        let conn = datasource::get_connection()
            .map_err(|err|format!("Can not connect to oracle: {}", err))?;

        println!("DynamicQuery.execute_query,sql: {}", &self.sql);

        let results_provider = Box::new( DynamicResultsProvider { columns: self.columns, column_names: self.column_names } );
        let params_provider = Box::new( DynamicParamsProvider { columns: self.param_columns });

        let stmt = conn.prepare_dynamic(&self.sql, params_provider)
            .map_err(|err|format!("Can not prepare statement: {}", err))?;

        let query = stmt.query_dynamic(results_provider, 1)
            .map_err(|err|format!("Can not create query from statement: {}", err))?;

        let result = query.fetch_one(self.parsed_params)
            .map_err(|err|format!("Can not fetch row by pk: {}", err))?;

        Ok( format!("[{}]", result) )
    }
}

impl ParsedParameter {
    fn parse(tp: oracle::SqlType, value: String) -> Result<Self, &'static str> {
        match tp {
            oracle::SqlType::Int16 => {
                let val: i16 = value.parse().unwrap();
                Ok(ParsedParameter::Int16(val))
            },
            oracle::SqlType::Int32 => {
                let val: i32 = value.parse().unwrap();
                Ok(ParsedParameter::Int32(val))
            },
            oracle::SqlType::Int64 => {
                let val: i64 = value.parse().unwrap();
                Ok(ParsedParameter::Int64(val))
            },
            oracle::SqlType::Varchar => {
                Ok(ParsedParameter::Varchar(value))
            },
            _ => Err("Not supported type for Primary key")
        }
    }

    fn project_value(&self, p: &mut oracle::ParamValue) {
        match self {
            Self::Int16(val) => {
                val.project_value(p);
            },
            Self::Int32(val) => {
                val.project_value(p);
            },
            Self::Int64(val) => {
                val.project_value(p);
            },
            Self::Varchar(val) => {
                val.project_value(p);
            },
        };
    }
}

impl oracle::ResultsProvider<String> for DynamicResultsProvider {
    fn sql_descriptors(&self) -> Vec<oracle::TypeDescriptor> {
        self.columns.iter().map(|c|c.oci_data_type.clone()).collect()
    }

    fn gen_result(&self, rs: oracle::ResultSet) -> String {
        let results: Vec<String> = self.columns
            .iter()
            .zip(self.column_names.iter())
            .zip(rs.iter())
            .map(|((c, name), value)|{
                let result = value.to_owned().try_to_string(&c.col_type).unwrap_or_else(|err| err.to_string());
                format!("\"{}\":{}", name, result)
            }).collect();

        format!("{{ {} }}", results.join(","))
    }
}

impl oracle::ParamsProvider<Vec<ParsedParameter>> for DynamicParamsProvider {
    fn members(&self) -> Vec<oracle::Member> {
        self.columns.iter()
            .map(|c| {
                oracle::Member::new(c.oci_data_type, oracle::Identifier::Unnamed)
            })
            .collect()
    }

    fn project_values(&self, params: &Vec<ParsedParameter>, projecton: &mut oracle::ParamsProjection) {
        for (idx,param) in params.iter().enumerate() {
            let p = unsafe { projecton.get_unchecked_mut(idx) };
            param.project_value(p);
        }
    }
}
