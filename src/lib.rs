#![allow(unused, dead_code)]

pub mod cli;

pub mod nomming {

    use std::{
        fs::{
            ReadDir,
            DirEntry,
            read_dir,
        },
        path::{
            Path,
            PathBuf,
        },
    };
    use nom::{
        IResult,
        multi::*,
        branch::*,
        sequence::*,
        character::*,
        combinator::*,
        bytes::complete::*,
    };
    use anyhow::{Result, bail, anyhow};
    use thiserror::Error;
    use serde_json::{Map, Value};

    #[derive(Debug)]
    pub struct Documents {
        templates: ReadDir,
        reports: ReadDir,
        resources: ReadDir,
    }

    impl Documents {
        fn new(path: &PathBuf) -> Result<Self> {
            let templates = path
                .join("Templates")
                .read_dir()?;
            let reports = path
                .join("Reports")
                .read_dir()?;
            let resources = path
                .join("Resources")
                .read_dir()?;
            Ok(Self{templates, reports, resources})
        }
    }


    #[derive(Debug, Clone)]
    pub struct Buffer {
        projdata: Value,
        userdata: Value,
        partdata: Value,
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
    impl Buffer {
        pub fn new(json: Map<String, Value>) -> Result<Self> {
            let projdata = json
                .get("projdata")
                .ok_or(MissingFieldError::ProjData)?
                .clone();
            let userdata = json
                .get("userdata")
                .ok_or(MissingFieldError::UserData)?
                .clone();
            let partdata = json
                .get("partdata")
                .ok_or(MissingFieldError::PartData)?
                .clone();
            Ok(Self{projdata, userdata, partdata})
        }
        pub fn projdata (&self) -> Result<&Map<String, Value>> {
            if let Value::Object(projdata) = &self.projdata {
                Ok(projdata)
            } else {
                bail!(
                    r"expeted project data to be a json obect
                    found {:#?}",
                    self.projdata
                    )
            }
        }
        pub fn userdata (&self) -> Result<&Map<String, Value>> {
            if let Value::Object(projdata) = &self.userdata {
                Ok(projdata)
            } else {
                bail!(
                    r"expeted user data to be a json obect
                    found {:#?}",
                    self.userdata
                    )
            }
        }
        pub fn partdata (&self) -> Result<&Map<String, Value>> {
            if let Value::Object(projdata) = &self.partdata {
                Ok(projdata)
            } else {
                bail!(
                    r"expeted part data to be a json obect
                    found {:#?}",
                    self.partdata
                    )
            }
        }
        pub fn headers (&self) -> Result<&Vec<Value>> {
            if let Some(Value::Array(headers)) = self.partdata()?.get("headers") {
                    Ok(headers)
            } else {
                bail!(
                    r"expeted headers to be a json array
                    found {:#?}",
                    self.partdata
                    )
            }
        }
        pub fn listed_reports (&self) -> Result<Vec<&str>> {
            let is_reports = |value| value == "rep";
            let is_not_headers = |key| key != "headers";
            let missing_rep_index = anyhow!(
                "headers array does not contain rep variable"
                );
            let empty_rep_data = anyhow!(
                "partdata does not contain reports array"
                );
            let reports_index = self
                .headers()?
                .into_iter()
                .position(|header| is_reports(header))
                .ok_or(missing_rep_index)?;
            let mut reports = self
                .partdata()?
                .into_iter()
                .filter(|&(key, value)| is_not_headers(key))
                .filter_map(|(key, value)| 
                            if let Value::Array(data) = value {
                                data.get(reports_index)
                            } else { None }
                           )
                .filter_map(|value|
                            if let Value::Array(reports) = value {
                                Some(reports)
                            } else { None }
                           )
                .flatten()
                .filter_map(|report| report.as_str() )
                .collect::<Vec<_>>();
            reports.sort_unstable();
            reports.dedup();
            if reports.is_empty() {
                Err(empty_rep_data)
            } else {
                Ok(reports)
            }
        }
    }


    fn is_html(p: &PathBuf) -> bool {
        if let Some(extension) = p.extension() {
            extension == "html"
        } else { false }
    }
}

