#![allow(unused, dead_code)]

pub mod cli;

pub mod nomming {

    use serde::{ Serialize, Deserialize, };
    use anyhow::{ Result, anyhow, bail, };
    use serde_json::{ Map, Value, json, };
    use rayon::prelude::*;

    use std::{
        cmp::max,
        sync::Arc,
        path::PathBuf,
        fs::{ File, ReadDir, read_dir, read_to_string, },
    };
    use nom::{
        IResult,
        sequence::{ pair, tuple, preceded, },
        bytes::complete::{ tag, take_until, },
    };





    #[derive(Debug, Clone)]
    pub struct Documents<'b> {
        pub root: &'b PathBuf,
        pub reports: Box<[PathBuf]>,
        pub templates: Box<[PathBuf]>,
        pub resources: Box<[PathBuf]>,
    }
    impl<'b> Documents<'b> {
        fn new(root: &'b PathBuf) -> Result<Self> {
            let reports = read_dir(root.join("Reports"))?
                .filter_map( |entry| entry.ok())
                .map( |entry| entry.path())
                .collect();
            let templates = read_dir(root.join("Templates"))?
                .filter_map( |entry| entry.ok())
                .map( |entry| entry.path())
                .collect();
            let resources = read_dir(root.join("resources"))?
                .filter_map( |entry| entry.ok())
                .map( |entry| entry.path())
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
        pub fn index_part_headers(&self, value: Value) -> Option<usize> {
            if let Some(Value::Array(headers)) = self.partdata.get("headers") {
                headers.iter().position(|v| *v == value)
            } else { None }
        }
        pub fn list_all_reports(&self) -> Result<Vec<&str>> {
            let reports_index = self
                .index_part_headers(json!("rep"))
                .ok_or(anyhow!("\"rep\" header not found"))?;
            let mut listed_reports = self
                .partdata
                .iter()
                .filter( |&(key, value)| key != "headers" )
                .filter_map( |(key, value)| value.as_array() )
                .filter_map( |entries| entries.get(reports_index) )
                .filter_map( |reports| reports.as_array() )
                .flatten()
                .filter_map( |report| report.as_str() )
                .collect::<Vec<_>>();
            listed_reports.dedup();
            Ok(listed_reports)
        }
    }





    #[derive(Debug, Clone)]
    pub struct Template<'b> {
        body: &'b str,
        title_block: &'b str,
        data_block: &'b str,
        sort_by_row: &'b str,
        pattern_row: &'b str,
    }
    impl<'b> Template<'b> {
        pub fn new(s: &'b str) -> IResult<&str, Self> {
            let (_, body) = Self::body(s)?;
            let (_, (title_block, data_block)) = Self::blocks(s)?;
            let (_, (sort_by_row, pattern_row)) = Self::rows(s)?;
            Ok((s, Self{
                body,
                title_block,
                data_block,
                sort_by_row,
                pattern_row,
            }))
        }
        fn blocks(s: &str) -> IResult<&str, (&str, &str)> {
            tuple((
                    Self::title_block,
                    Self::data_block,
                    ))(s)
        }
        fn rows(s: &str) -> IResult<&str, (&str, &str)> {
            tuple((
                    Self::sort_by_row,
                    Self::pattern_row,
                    ))(s)
        }
        fn body(s: &str) -> IResult<&str, &str> {
            preceded(
                pair(
                    take_until("<body>"),
                    tag("<body>")
                    ),
                    take_until("</body>")
                    )(s)
        }
        fn title_block(s: &str) -> IResult<&str, &str> {
            preceded(
                pair(
                    take_until("<table class=\"title_block\">"),
                    tag("<table class=\"title_block\">")
                    ),
                    take_until("</table>")
                    )(s)
        }
        fn data_block(s: &str) -> IResult<&str, &str> {
            preceded(
                pair(
                    take_until("<table class=\"data_block\">"),
                    tag("<table class=\"data_block\">")
                    ),
                    take_until("</table>")
                    )(s)
        }
        fn sort_by_row(s: &str) -> IResult<&str, &str> {
            preceded(
                pair(
                    take_until("<tr class=\"sort_by_row\">"),
                    tag("<tr class=\"sort_by_row\">")
                    ),
                    take_until("</tr>")
                    )(s)
        }
        fn pattern_row(s: &str) -> IResult<&str, &str> {
            preceded(
                pair(
                    take_until("<tr class=\"pattern_row\">"),
                    tag("<tr class=\"pattern_row\">")
                    ),
                    take_until("</tr>")
                    )(s)
        }
    }



    #[derive(Debug, Clone)]
    pub struct FileDispatch<'b> {
        buffer: &'b Buffer,
        documents: &'b Documents<'b>,
    }
    impl<'b> FileDispatch<'b> {
        pub fn new(buffer: &'b Buffer, documents: &'b Documents) -> Self {
            Self{buffer, documents}
        }
        pub fn dispatch(&self) -> Result<Box<[String]>> {
            let templates = self
                .buffer
                .list_all_reports()?
                .iter()
                .filter_map( |&stem| self.documents.check_template(stem))
                .par_bridge()
                .filter_map( |path| read_to_string(path).ok())
                .collect();
            Ok(templates)
        }
    }
}

