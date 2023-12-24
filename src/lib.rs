#![allow(unused, dead_code)]

pub mod cli;

pub mod nomming {

    use rayon::prelude::*;
    use anyhow::{ anyhow, Result, };
    use serde::{ Serialize, Deserialize, };
    use serde_json::{ json, Map, Value, };

    use std::{
        cmp::max,
        io::Write,
        path::PathBuf,
        fs::{ OpenOptions, read_dir, read_to_string, canonicalize, },
    };
    use nom::{
        IResult,
        combinator::recognize,
        sequence::{ pair, tuple, preceded, },
        bytes::complete::{ tag, take_until, },
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
                .filter_map( |entry| entry.ok())
                .map( |entry| entry.path())
                .collect();
            let templates = read_dir(root.join("Templates"))?
                .filter_map( |entry| entry.ok())
                .map( |entry| entry.path())
                .collect();
            let resources = read_dir(root.join("Resources"))?
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
            let mut listed_reports = self.partdata
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
    #[derive(Debug, Clone)]
    enum Block {
        Tr(&'static str),
        Td(&'static str),
        Th(&'static str),
        Thead(&'static str),
        Tfoot(&'static str),
        Table(&'static str),
    }
    impl<'b> Block {
        fn open_tag(&self) -> String {
            match self {
                Self::Th(class) => format!("<th class=\"{class}\""),
                Self::Tr(class) => format!("<tr class=\"{class}\""),
                Self::Td(class) => format!("<td class=\"{class}\""),
                Self::Table(class) => format!("<table class=\"{class}\""),
                Self::Thead(class) => format!("<thead class=\"{class}\""),
                Self::Tfoot(class) => format!("<tfoot class=\"{class}\""),
            }
        }
        fn close_tag(&self) -> &'static str {
            match self {
                Self::Th(_) => "</th>",
                Self::Tr(_) => "</tr>",
                Self::Td(_) => "</td>",
                Self::Table(_) => "</table>",
                Self::Thead(_) => "</thead>",
                Self::Tfoot(_) => "</tfoot>",
            }
        }
        fn parse(&self, s: &'b str) -> IResult<&'b str, &'b str> {
            preceded(
                take_until(self.open_tag().as_str()),
                recognize(pair(
                    take_until(self.close_tag()),
                    tag(self.close_tag()),
                    )),
                    )(s)
        }
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
        pub fn sort_by_row(s: &str) -> IResult<&str, &str> {
            Block::Tr("sort_by_row").parse(s)
        }
        pub fn pattern_row(s: &str) -> IResult<&str, &str> {
            Block::Tr("pattern_row").parse(s)
        }
        pub fn title_block(s: &str) -> IResult<&str, &str> {
            Block::Table("title_block").parse(s)
        }
        pub fn data_block(s: &str) -> IResult<&str, &str> {
            Block::Table("data_block").parse(s)
        }
        pub fn body(s: &str) -> IResult<&str, &str> {
            preceded(
                take_until("<body>"),
                recognize(pair( take_until("</body>"), tag("</body>"))),
                )(s)
        }
        pub fn blocks(s: &str) -> IResult<&str, (&str, &str)> {
            tuple((
                    Self::title_block,
                    Self::data_block,
                    ))(s)
        }
        pub fn rows(s: &str) -> IResult<&str, (&str, &str)> {
            tuple((
                    Self::sort_by_row,
                    Self::pattern_row,
                    ))(s)
        }
    }






    #[derive(Debug, Clone)]
    pub struct Dispatch<'b> {
        buffer: &'b Buffer,
        documents: &'b Documents,
        batch: TemplateBatch<'b>,
        log: Option<PathBuf>,
    }
    #[derive(Debug, Clone)]
    enum TemplateBatch<'b> {
        Empty,
        Raw(Box<[String]>),
        Parsed(Box<[Template<'b>]>),
    }
    impl<'b> TemplateBatch<'b> {
        fn process(&'b self) -> Self {
            if let Self::Raw(batch) = self {
                Self::Parsed(
                    batch 
                    .par_iter()
                    .filter_map( |template| Template::new(template).ok())
                    .map( |(input, output)| output)
                    .collect())
            } else { self.clone() }
        }
    }
    impl<'b> Dispatch<'b> {
        pub fn new(buffer: &'b Buffer, documents: &'b Documents) -> Self {
            let log = None;
            let batch = TemplateBatch::Empty;
            Self{buffer, documents, batch, log}
        }
        pub fn with_log(mut self, log: PathBuf) -> Self {
            self.log = Some(log);
            return self
        }
        pub fn read_all(mut self) -> Result<Self> {
            self.batch = TemplateBatch::Raw(
                self.buffer 
                .list_all_reports()? 
                .iter() 
                .filter_map( |&stem| self.documents.check_template(stem)) 
                .inspect( |path| self.log(path))
                .par_bridge()
                .filter_map( |path| read_to_string(path).ok())
                .collect()
                );
            Ok(self)
        }
        pub fn parse_all(&'b self) -> Self {
            let mut processed_dispatch = self.clone();
            processed_dispatch.batch = self.batch.process();
            processed_dispatch
        }
        fn log(&self, template: &PathBuf) {
            if let Some(log) = &self.log {
                println!("{template:#?}");
                OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(log)
                    .expect("failed to open log")
                    .write(format!("{template:#?}\n").as_bytes())
                    .expect("failed to write to log");
            }
        }
    }
}
