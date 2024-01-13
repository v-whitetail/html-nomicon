use std::{
    ops::Deref,
    sync::Arc,
    collections::BTreeMap,
};
use anyhow::{ Result, anyhow, };
use serde::{ Serialize, Deserialize };

pub type Key = Arc<str>;
pub type List = Box<[Value]>;
pub type Value = Arc<str>;
pub type MixedList = Box<[Variable]>;
pub type UserData = BTreeMap<Key, Value>;
pub type ProjectData = BTreeMap<Key, Value>;


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
    pub projdata: ProjectData,
    pub userdata: UserData,
    pub partdata: PartData,
    #[serde(skip_deserializing)]
    reports: List,
    #[serde(skip_deserializing)]
    sort_index: Option<usize>,
    #[serde(skip_deserializing)]
    sort_values: List,
    #[serde(skip_deserializing)]
    part_map: BTreeMap<Key, MixedList>,
}
impl Buffer {
    fn reports(mut self) -> Result<()> {
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
        self.reports = listed_reports.into();
        Ok(())
    }
    fn sort_variable(mut self, sort_variable: Option<&str>) -> Result<()> {
        if let Some(index) = sort_variable.and_then(
            |variable| self.index_part_headers(variable)
            ) {
            self.sort_index = Some(index)
        };
        Ok(())
    }
    fn sort_values(mut self) -> Result<()> {
        if let Some(sort_index) = self.sort_index {
            let mut sort_values = self.partdata.parts
                .iter()
                .filter_map( |(_, value)| value.get(sort_index) )
                .filter_map( |variable| variable.as_name() )
                .collect::<Vec<_>>();
            sort_values.sort();
            sort_values.dedup();
            self.sort_values = sort_values.into_iter().collect();
        };
        Ok(())
    }
    fn part_map(mut self) -> Result<()> {
        if let Some(sort_index) = self.sort_index {
            let mut map = self.partdata.parts
                .iter()
                .filter_map(
                    |(key, value)| value.get(sort_index).and_then(
                        |sort_value| sort_value.as_name().and_then(
                            |sort_value| Some((sort_value, key))
                            )
                        )
                    )
                .collect::<Vec<_>>();
            map.sort();
            map.group_by( |(lhs, _), (rhs, _)| lhs == rhs ).map(
                |sort_list| sort_list.into_iter().filter_map(
                    |&(_, part_id)| self.partdata.parts.get(part_id.deref())
                    ).collect::<Box<[_]>>()
                )
                .zip(self.sort_values.iter())
                .map( |(part_ids, sort_value)| (sort_value, part_ids) )
                .collect::<BTreeMap<_, _>>();
        };
        Ok(())
    }
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
}
