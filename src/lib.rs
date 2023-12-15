#![allow(unused, dead_code)]

pub mod cli;

pub mod nomming {

    use std::path::PathBuf;
    use walkdir::WalkDir;
    use nom::{
        IResult,
        sequence::{
            pair,
            tuple,
            preceded,
        },
        bytes::complete::{
            tag,
            take_until,
        },
    };
    use anyhow::{Result, bail, anyhow};
    use thiserror::Error;
    use serde_json::{Map, Value};





    #[derive(Debug)]
    pub struct Documents {
        templates: WalkDir,
        reports: WalkDir,
        resources: WalkDir,
    }
    impl Documents {
        fn new(path: &PathBuf) -> Result<Self> {
            let templates = WalkDir::new(path.join("Templates"))
                .max_depth(1);
            let reports = WalkDir::new(path.join("Reports"))
                .max_depth(1);
            let resources = WalkDir::new(path.join("Resources"))
                .max_depth(2);
            Ok(Self{templates, reports, resources})
        }
    }





    #[derive(Debug, Clone)]
    pub struct Buffer<'b> {
        projdata: &'b Value,
        userdata: &'b Value,
        partdata: &'b Value,
    }
    #[derive(Debug, Error)]
    pub enum MissingFieldError{
        #[error("missing project data")]
        ProjData,
        #[error("missing user data")]
        UserData,
        #[error("missing part data")]
        PartData,
    }
    impl<'b> Buffer<'b> {
        pub fn new(json: &'b Map<String, Value>) -> Result<Self> {
            let projdata = json
                .get("projdata")
                .ok_or(MissingFieldError::ProjData)?;
            let userdata = json
                .get("userdata")
                .ok_or(MissingFieldError::UserData)?;
            let partdata = json
                .get("partdata")
                .ok_or(MissingFieldError::PartData)?;
            Ok(Self{projdata, userdata, partdata})
        }
        pub fn projdata (&self) -> Result<&Map<String, Value>> {
            if let Value::Object(projdata) = &self.projdata {
                Ok(projdata)
            } else {
                bail!(
                    "expeted project data to be a json obect\n\tfound:\n{:#?}",
                    self.projdata
                    )
            }
        }
        pub fn userdata (&self) -> Result<&Map<String, Value>> {
            if let Value::Object(projdata) = &self.userdata {
                Ok(projdata)
            } else {
                bail!(
                    "expeted user data to be a json obect\n\tfound:\n{:#?}",
                    self.userdata
                    )
            }
        }
        pub fn partdata (&self) -> Result<&Map<String, Value>> {
            if let Value::Object(projdata) = &self.partdata {
                Ok(projdata)
            } else {
                bail!(
                    "expeted part data to be a json obect\n\tfound:\n{:#?}",
                    self.partdata
                    )
            }
        }
        pub fn headers (&self) -> Result<&Vec<Value>> {
            if let Some(Value::Array(headers)) = self.partdata()?.get("headers") {
                Ok(headers)
            } else {
                bail!(
                    "expeted headers to be a json array\n\tfound:\n{:#?}",
                    self.partdata
                    )
            }
        }
        pub fn listed_reports(&'b self) -> Result<Vec<&str>> {

            let is_reports = |value| value == "rep";
            let is_not_headers = |key| key != "headers";
            let get_reports_array = |value: &'b Value, index|
                if let Value::Array(data) = value { data.get(index) }
                else { None };
            let get_listed_reports = |value: &'b Value|
                if let Value::Array(reports) = value { Some(reports) }
                else { None };
            let no_listed_reports = anyhow!("partdata does not contain reports array");
            let missing_report_header = anyhow!("headers array does not contain \"rep\" variable");


            let reports_index = self
                .headers()?
                .into_iter()
                .position(|header| is_reports(header))
                .ok_or(missing_report_header)?;
            let mut reports = self
                .partdata()?
                .into_iter()
                .filter(|&(key, value)| is_not_headers(key))
                .filter_map(|(key, value)| get_reports_array(value, reports_index))
                .filter_map(|value| get_listed_reports(value))
                .flatten()
                .filter_map(|report| report.as_str() )
                .collect::<Vec<_>>();

            reports.sort_unstable();
            reports.dedup();

            if reports.is_empty() { Err(no_listed_reports) }
            else { Ok(reports) }
        }
    }





    #[derive(Debug, Clone)]
    pub struct Template<'t> {
        body: &'t str,
        title_block: &'t str,
        data_block: &'t str,
        sort_by_row: &'t str,
        pattern_row: &'t str,
    }
    impl<'t> Template<'t> {
        pub fn new(s: &'t str) -> IResult<&str, Self> {
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



    pub struct BatchProcessor<'b> {
        buffer: Buffer<'b>,
        documents: Documents,
    }
    impl<'b> BatchProcessor<'b> {
        pub fn new(json: &'b Map<String, Value>, path: &PathBuf) -> Result<Self> {

            let buffer = Buffer::new(json)?;
            let documents = Documents::new(path)?;
            buffer.listed_reports()?;

            Ok(Self{buffer, documents})
        }
    }
}

