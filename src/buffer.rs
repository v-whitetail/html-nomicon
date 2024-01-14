use std::{
    ops::Deref,
    sync::Arc,
    collections::HashMap,
};
use rayon::prelude::*;
use serde::{ Serialize, Deserialize };
use anyhow::{ Result, anyhow, };

pub type Key = Arc<str>;
pub type List = Box<[Value]>;
pub type Value = Arc<str>;
pub type MixedList = Box<[Variable]>;
pub type UserData = HashMap<Key, Value>;
pub type ProjectData = HashMap<Key, Value>;


#[derive(Debug, Serialize, Deserialize)]
pub struct PartData {
    pub headers: List,
    pub parts: HashMap<Key, MixedList>,
}


#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    pub fn is_name(&self) -> bool {
        match self {
            Self::Name(_) => true,
            _ => false,
        }
    }
    pub fn is_list(&self) -> bool {
        match self {
            Self::List(_) => true,
            _ => false,
        }
    }
}






#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Buffer {
    pub projdata: Arc<ProjectData>,
    pub userdata: Arc<UserData>,
    pub partdata: Arc<PartData>,
    #[serde(skip_deserializing)]
    sort_index: Option<usize>,
    #[serde(skip_deserializing)]
    part_map: Option<HashMap<Value, Vec<Key>>>,
}
impl Buffer {
    pub fn list_all_reports(&self) -> Result<Box<[Value]>> {
        let reports_index = self
            .index_part_headers("rep")?;
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
    pub fn sort(self, sort_variable: Option<&str>) -> Result<Self> {
        let sorted = self.clone()
            .sort_index(sort_variable)?
            .part_map_keys()?
            .part_map_values()?;
        Ok(sorted)
    }
    fn index_part_headers(&self, value: &str) -> Result<usize> {
        self.partdata.headers
            .iter()
            .position( |v| **v == *value )
            .ok_or_else( || anyhow!("\"{value:#?}\" not found in headers") )
    }
    fn sort_index(mut self, sort_variable: Option<&str>) -> Result<Self> {
        if let Some(index) = sort_variable.and_then(
            |variable| self.index_part_headers(variable).ok()
            ) {
            self.sort_index = Some(index);
        };
        Ok(self)
    }
    fn part_map_keys(mut self) -> Result<Self> {
        if let Some(sort_index) = self.sort_index {
            let mut sort_values = self.partdata.parts
                .iter()
                .filter_map( |(_, value)| value.get(sort_index) )
                .filter_map( |variable| variable.as_name() )
                .collect::<Vec<_>>();
            sort_values.sort();
            sort_values.dedup();
            let part_map = sort_values
                .into_iter()
                .fold( HashMap::new(), |mut part_map, sort_key| {
                    part_map.insert(sort_key, Vec::new());
                    part_map
                });
            self.part_map = Some(part_map);
        };
        Ok(self)
    }
    fn part_map_values(mut self) -> Result<Self> {
        if let Some(ref mut part_map) = self.part_map {
            part_map
                .par_iter_mut()
                .for_each( |(map_key, mut map_values)| {
                    self.partdata.parts
                        .iter()
                        .for_each( |(part_key, part_values)| {
                            if part_values.contains(&Variable::Name(map_key.clone())) {
                                map_values.push(part_key.clone());
                            };
                        })
                });
        }
        Ok(self)
    }
}
