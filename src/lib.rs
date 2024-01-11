#![allow(unused, dead_code)]

pub mod cli;
pub mod nomming;
pub mod processing {

    use nom::IResult;
    use rayon::prelude::*;
    use anyhow::{ anyhow, Result, };
    use serde_json::{ json, };
    use serde::{ Serialize, Deserialize, };
    use crate::{ cli::Input, nomming::*, buffer::*, };
    use std::{
        cmp::max,
        ops::Deref,
        sync::Arc,
        path::PathBuf,
        borrow::Borrow,
        collections::*,
        fs::{ OpenOptions, read_dir, read_to_string, canonicalize, },
    };






    #[derive(Debug, Clone)]
    pub struct Documents {
        pub root: PathBuf,
        pub reports: Box<[PathBuf]>,
        pub templates: Box<[PathBuf]>,
        pub resources: Box<[PathBuf]>,
    }
    impl Documents {
        pub fn new(path: &PathBuf) -> Result<Self> {
            let root = path.canonicalize()?;
            let reports = read_dir(root.join("Reports"))?
                .filter_map( |entry| entry.ok() )
                .map( |entry| entry.path() )
                .collect();
            let templates = read_dir(root.join("Templates"))?
                .filter_map( |entry| entry.ok() )
                .map( |entry| entry.path() )
                .collect();
            let resources = read_dir(root.join("Resources"))?
                .filter_map( |entry| entry.ok() )
                .map( |entry| entry.path() )
                .collect();
            Ok(Self{root, templates, reports, resources})
        }
        fn check_template(&self, stem: &str) -> Option<PathBuf> {
            let template_path = self.root
                .join("Templates")
                .join(stem)
                .with_extension("html");
            let is_present = self.templates
                .into_iter()
                .fold(false, |is_present, path|
                      max(template_path == *path, is_present));
            if is_present { return Some(template_path)}
            else { return None }
        }
    }











    #[derive(Debug, Clone)]
    pub struct Template<'b> {
        body: &'b str,
        data_block: &'b str,
        title_block: &'b str,
        sorting_row: Option<&'b str>,
        pattern_row: Option<&'b str>,
    }
    impl<'b> Template<'b> {
        pub fn new(s: &'b str) -> IResult<&str, Self> {
            let (_, body) = body(s)?;
            let (_, (title_block, data_block)) = blocks(body)?;
            let (_, (sorting_row, pattern_row)) = rows(data_block)?;
            Ok((s, Self{
                body,
                data_block,
                title_block,
                sorting_row,
                pattern_row,
            }))
        }
    }






    #[derive(Debug, Clone)]
    pub struct RawTemplates {
        listed_reports: Box<[Value]>,
        templates: Box<[String]>,
    }
    impl<'b> RawTemplates {
        pub fn new(buffer: &'b Buffer, documents: &Documents) -> Result<Self> {
            let listed_reports = buffer.list_all_reports()?;
            let templates = listed_reports
                .par_iter() 
                .filter_map( |stem| documents.check_template(stem) )
                .filter_map( |path| read_to_string(path).ok() )
                .collect();
            Ok(Self{listed_reports, templates})
        }
    }






    #[derive(Debug, Clone)]
    pub struct ParsedTemplates<'b>{
        buffer: &'b Buffer,
        templates: Box<[Template<'b>]>,
    }
    impl<'b> ParsedTemplates<'b> {
        pub fn new(
            buffer: &'b Buffer,
            raw_templates: &'b RawTemplates
            ) -> Result<Self> {
            let templates = raw_templates
                .templates
                .par_iter()
                .filter_map( |template| Template::new(template).ok() )
                .map( |(input, output)| output )
                .collect();
            Ok(Self{buffer, templates})
        }
    }
}





pub mod buffer {
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
}
