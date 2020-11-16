use std::collections::HashMap;
use std::rc::Rc;

pub struct MetaInfo {
    pub schemas:  HashMap<Rc<String>,SchemaInfo>,
}

pub struct SchemaInfo {
    pub name:    Rc<String>,
    pub tables:  HashMap<Rc<String>,TableInfo>,
}

pub struct TableInfo {
    pub name:        Rc<String>,
    pub is_view:     bool,
    pub temporary:   bool,
    pub num_rows:    i32,
    pub columns:     Vec<ColumnInfo>,
    pub primary_key: Option<PrimaryKey>,
    pub indexes:     Vec<Index>
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColumnType {
    Int16, Int32, Int64, Float64, Varchar, DateTime, Blob, Clob, Long, Unsupported
}

pub struct ColumnInfo {
    pub name:           String,
    pub col_type:       ColumnType,
    pub col_type_name:  String,
    pub oci_data_type:  u16,
    pub col_len:        u16,
    pub nullable:       bool,
    pub data_precision: u16,
    pub data_scale:     u16,
    pub buffer_len:     usize
}

pub struct PrimaryKey {
    pub name:    String,
    pub columns: Vec<String>
}

pub struct Index {
    pub name:    String,
    pub unique:  bool,
    pub columns: Vec<IndexColumn>
}

pub struct IndexColumn {
    pub name: String,
    pub desc: bool
}
