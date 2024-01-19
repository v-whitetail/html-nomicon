use std::{
    ops::Deref,
    sync::Arc,
    borrow::Borrow,
    collections::*,
};
use rayon::prelude::*;
use serde::{ Serialize, Deserialize };
use anyhow::{ Result, anyhow, };

type PyStr = Arc<str>;
type PyInt = Arc<isize>;
type PyList = Vec<PyValue>;
type PySet = BTreeSet<PyStr>;
type PyDict = BTreeMap<PyStr, PyValue>;

#[derive(Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Clone)]
#[serde(untagged)]
enum PyValue {
    Str(PyStr),
    Int(PyInt),
    List(PyList),
    Set(PySet),
    Dict(PyDict),
}
impl PyValue {
    fn is_str(&self)  -> bool { if let PyValue::Str(_)  = self { true } else { false } }
    fn is_int(&self)  -> bool { if let PyValue::Int(_)  = self { true } else { false } }
    fn is_list(&self) -> bool { if let PyValue::List(_) = self { true } else { false } }
    fn is_set(&self)  -> bool { if let PyValue::Set(_)  = self { true } else { false } }
    fn is_dict(&self) -> bool { if let PyValue::Dict(_) = self { true } else { false } }
    fn to_str(&self)  -> Option<PyStr>  { if let PyValue::Str(inner)  = self { Some(inner.clone()) } else { None } }
    fn to_int(&self)  -> Option<PyInt>  { if let PyValue::Int(inner)  = self { Some(inner.clone()) } else { None } }
    fn to_list(&self) -> Option<PyList> { if let PyValue::List(inner) = self { Some(inner.clone()) } else { None } }
    fn to_set(&self)  -> Option<PySet>  { if let PyValue::Set(inner)  = self { Some(inner.clone()) } else { None } }
    fn to_dict(&self) -> Option<PyDict> { if let PyValue::Dict(inner) = self { Some(inner.clone()) } else { None } }
    fn index_list(&self, i: usize) -> Option<&Self> {
        if let Self::List(inner) = self { inner.get(i) } else { None }
    }
    fn insert_list(&mut self, v: Self) -> bool {
        if let Self::List(inner) = self { inner.push(v); true } else { false }
    }
}

#[derive(Debug, Deserialize)]
struct PartData {
    headers: PyList,
    groups: PyDict,
    parts: PyDict,
}
impl PartData {
    fn group_by_header(&self, h: &str) -> Result<PyDict> {
        let header_index = self.headers.iter()
            .position( |v| PyValue::Str(h.into()) == *v )
            .ok_or_else(|| anyhow!("\"{h}\" not found in headers") )?;
        let sorted = self.parts.iter()
            .fold( PyDict::new(), |mut dict, (key, value)| {
                let sort_value = value
                    .index_list(header_index)
                    .and_then(
                    |value| value.to_str()).unwrap_or_else(||key.clone());
                if let Some(prev) = dict.get_mut(&sort_value) {
                    prev.insert_list(PyValue::Str(key.clone()));
                } else {
                    dict.insert(sort_value, PyValue::Str(key.clone()));
                };
                dict
            });
        return Ok(sorted)
    }
}

#[derive(Debug, Deserialize)]
pub struct Buffer {
    project_data: PyDict,
    user_data: PyDict,
    part_data: PartData,
}
impl Buffer {
    pub fn globals(&self) -> Box<[(PyStr, PyStr)]> {
        self.project_data
            .iter().chain(self.user_data.iter())
            .filter_map(
                |(key, value)| value.to_str().and_then(
                    |v| Some((key.to_owned(), v))
                    ))
            .collect()
    }
}
