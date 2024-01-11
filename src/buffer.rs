use std::{
    ops::Deref,
    sync::Arc,
    borrow::Borrow,
    collections::BTreeMap,
};
use anyhow::{ Result, anyhow, };
use serde::{ Serialize, Deserialize };

pub type Key = Box<str>;
pub type List = Box<[Value]>;
pub type Value = Box<str>;
pub type MixedList = Box<[Variable]>;


#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectData (
    BTreeMap<Key, Value>,
    );


#[derive(Debug, Serialize, Deserialize)]
pub struct UserData (
    BTreeMap<Key, Value>,
    );


#[derive(Debug, Serialize, Deserialize)]
pub struct PartData {
    pub headers: List,
    pub parts: BTreeMap<Key, MixedList>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Variable {
    Name(Value),
    List(List),
}
impl Variable {
    pub fn as_name(&self) -> Option<Value> {
        match self {
            Self::Name(value) => Some(value.clone()),
            _ => None,
        }
    }
    pub fn as_list(&self) -> Option<List> {
        match self {
            Self::List(list) => Some(list.clone()),
            _ => None,
        }
    }
}






#[derive(Debug, Serialize, Deserialize)]
pub struct Buffer {
    projdata: ProjectData,
    userdata: UserData,
    partdata: PartData,
}
impl Buffer {
    fn index_part_headers(&self, value: &str) -> Option<usize> {
        self.partdata.headers.iter().position(|v| **v == *value)
    }
    pub fn list_all_reports(&self) -> Result<Box<[Value]>> {
        let reports_index = self
            .index_part_headers("rep")
            .ok_or(anyhow!("\"rep\" header not found"))?;
        let mut listed_reports = self.partdata.parts
            .iter()
            .filter_map( |(_, value)| value.get(reports_index) )
            .filter_map( |reports| reports.as_list() )
            .map( |list| list.as_ref().to_owned() )
            .flatten()
            .collect::<Vec<_>>();
        listed_reports.sort();
        listed_reports.dedup();
        Ok(listed_reports.into())
    }
    pub fn list_parts(&self, sort: &str) -> Result<BTreeMap<Key, Value>> {
        let sort_index = self.index_part_headers(sort)
            .ok_or(anyhow!("\"{sort:#?}\" header not found"))?;
        let mut parts = self.partdata.parts
            .iter()
            .filter_map(
                |(part_id, value)|
                value
                .get(sort_index)
                .and_then(|sort_value| sort_value.as_name())
                .and_then(|sort_value| Some((sort_value, part_id.clone())))
                )
            .collect::<BTreeMap<_,_>>();
        Ok(parts)
    }
}
