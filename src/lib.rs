#![allow(unused, dead_code)]

pub mod cli;

pub mod nomming {

    use rayon::prelude::*;
    use anyhow::{ anyhow, Result, };
    use serde::{ Serialize, Deserialize, };
    use serde_json::{ json, Map, Value, };
    use nom::{
        IResult,
        combinator::recognize,
        sequence::{ pair, preceded, },
        bytes::complete::{ tag, take_until, },
    };
    use std::{
        cmp::max,
        io::Write,
        path::PathBuf,
        borrow::Cow,
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
        pub fn index_part_headers(&self, value: Value) -> Option<usize> {
            if let Some(Value::Array(headers)) = self.partdata.get("headers") {
                headers.iter().position(|v| *v == value)
            } else { None }
        }
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
            listed_reports.dedup();
            Ok(listed_reports)
        }
    }






    #[derive(Debug, Clone)]
    enum Block {
        Body,
        Tr(&'static str),
        Td(&'static str),
        Th(&'static str),
        Thead(&'static str),
        Tfoot(&'static str),
        Table(&'static str),
    }
    impl<'b> Block {
        fn open_tag(&self) -> Cow<str> {
            match self {
                Self::Body => format!("<body>"),
                Self::Th(class) => format!("<th class=\"{class}\""),
                Self::Tr(class) => format!("<tr class=\"{class}\""),
                Self::Td(class) => format!("<td class=\"{class}\""),
                Self::Table(class) => format!("<table class=\"{class}\""),
                Self::Thead(class) => format!("<thead class=\"{class}\""),
                Self::Tfoot(class) => format!("<tfoot class=\"{class}\""),
            }.into()
        }
        fn close_tag(&self) -> &'static str {
            match self {
                Self::Body => "</body>",
                Self::Th(_) => "</th>",
                Self::Tr(_) => "</tr>",
                Self::Td(_) => "</td>",
                Self::Table(_) => "</table>",
                Self::Thead(_) => "</thead>",
                Self::Tfoot(_) => "</tfoot>",
            }
        }
        fn parse(&self, s: &'b str) -> IResult<&'b str, &'b str> {
            let take_once = |t| recognize(pair(take_until(t),tag(t)));
            preceded(
                take_until(&*self.open_tag()),
                take_once(self.close_tag()),
                )(s)
        }
    }





    #[derive(Debug, Clone)]
    pub struct Template<'b> {
        body: &'b str,
        title_block: &'b str,
        data_block: &'b str,
        sorting_row: &'b str,
        pattern_row: &'b str,
    }
    impl<'b> Template<'b> {
        pub fn body(s: &str) -> IResult<&str, &str> {
            Block::Body.parse(s)
        }
        pub fn title_block(s: &str) -> IResult<&str, &str> {
            Block::Table("title_block").parse(s)
        }
        pub fn data_block(s: &str) -> IResult<&str, &str> {
            Block::Table("data_block").parse(s)
        }
        pub fn sorting_row(s: &str) -> IResult<&str, &str> {
            Block::Tr("sorting_row").parse(s)
        }
        pub fn pattern_row(s: &str) -> IResult<&str, &str> {
            Block::Tr("pattern_row").parse(s)
        }
        pub fn blocks(s: &str) -> IResult<&str, (&str, &str)> {
            pair( Self::title_block, Self::data_block )(s)
        }
        pub fn rows(s: &str) -> IResult<&str, (&str, &str)> {
            pair( Self::sorting_row, Self::pattern_row )(s)
        }
        pub fn new(s: &'b str) -> IResult<&str, Self> {
            let (_, body) = Self::body(s)?;
            let (_, (title_block, data_block)) = Self::blocks(s)?;
            let (_, (sorting_row, pattern_row)) = Self::rows(s)?;
            Ok((s, Self{
                body,
                data_block,
                title_block,
                sorting_row,
                pattern_row,
            }))
        }
        pub fn populate(&self, raw: &String, buffer: &'b Buffer) -> String {
            let raw = raw.to_owned();
            let title_block = self.title_block.to_owned();
            let sorting_row = self.sorting_row.to_owned();
            let pattern_row = self.pattern_row.to_owned();
            buffer.projdata.iter()
                .chain( buffer.userdata.iter() )
                .filter_map(
                    |(key, value)| value.as_str().and_then(
                        |value| Some((key, value)))
                    )
                .for_each( |(key, value)| {
                    title_block.replace(key, value);
                });
            let (sort_criteria, pattern_criteria):
                (Vec<&str>, Vec<&str>) = buffer
                 .list_all_headers()
                 .into_iter()
                 .partition( |&&header| self.sorting_row.contains(header) );


            raw.replace( self.title_block, &title_block );
            todo!()
        }
    }





    pub struct RawTemplates<'b>{
        listed_reports: Vec<&'b str>,
        templates: Box<[String]>,
    }
    impl<'b> RawTemplates<'b> {
        pub fn new(buffer: &'b Buffer, documents: &Documents) -> Result<Self> {
            let listed_reports = buffer.list_all_reports()?;
            let templates = listed_reports
                .iter() 
                .filter_map( |&stem| documents.check_template(stem) )
                .par_bridge()
                .filter_map( |path| read_to_string(path).ok() )
                .collect();
            Ok(Self{listed_reports, templates})
        }
    }





    pub struct ParsedTemplates<'b>{
        buffer: &'b Buffer,
        templates: Box<[Template<'b>]>,
    }
    impl<'b> ParsedTemplates<'b> {
        pub fn new(buffer: &'b Buffer, raw_templates: &'b RawTemplates<'b>)
            -> Result<Self> {
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
