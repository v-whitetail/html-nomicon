#![allow(unused, dead_code)]

pub mod cli;
pub mod nomming;
pub mod processing {

    use nom::IResult;
    use rayon::prelude::*;
    use anyhow::{ anyhow, Result, };
    use serde_json::{ json, Map, Value, };
    use serde::{ Serialize, Deserialize, };
    use crate::{ cli::Input, nomming::*, };
    use std::{
        cmp::max,
        path::PathBuf,
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






    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Buffer {
        projdata: Map<String, Value>,
        userdata: Map<String, Value>,
        partdata: Map<String, Value>,
    }
    impl Buffer {
        pub fn list_all_headers(&self) -> Box<[&str]> {
            self.partdata
                .iter()
                .filter( |&(key, value)| key == "headers" )
                .filter_map( |(key, value)| value.as_array() )
                .flatten()
                .filter_map( |header| header.as_str() )
                .collect()
        }
        pub fn list_all_reports(&self) -> Result<Vec<&str>> {
            let reports_index = self
                .index_part_headers(json!("rep"))
                .ok_or(anyhow!("\"rep\" header not found"))?;
            let mut listed_reports = self.partdata
                .iter()
                .filter( |&(key, value)| key != "headers" )
                .filter_map( |(key, value)| value.as_array() )
                .filter_map( |entries| entries.get(reports_index) )
                .filter_map( |reports| reports.as_array() )
                .flatten()
                .filter_map( |report| report.as_str() )
                .collect::<Vec<_>>();
            listed_reports.sort();
            listed_reports.dedup();
            dbg!(&listed_reports);
            Ok(listed_reports)
        }
        fn index_part_headers(&self, value: Value) -> Option<usize> {
            if let Some(Value::Array(headers)) = self.partdata.get("headers") {
                headers.iter().position(|v| *v == value)
            } else { None }
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
    pub struct RawTemplates<'b>{
        listed_reports: Vec<&'b str>,
        templates: Box<[String]>,
    }
    impl<'b> RawTemplates<'b> {
        pub fn new(buffer: &'b Buffer, documents: &Documents) -> Result<Self> {
            let listed_reports = buffer.list_all_reports()?;
            let templates = listed_reports
                .par_iter() 
                .filter_map( |&stem| documents.check_template(stem) )
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
            raw_templates: &'b RawTemplates<'b>
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
