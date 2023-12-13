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
        }
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
    use anyhow::{Result, bail};
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
        fn new(json: Map<String, Value>) -> Result<Self> {
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
        fn projdata (&self) -> Result<&Map<String, Value>> {
            if let Value::Object(projdata) = &self.projdata {
                Ok(projdata)
            } else {
                bail!("expeted project data to be a json obect")
            }
        }
        fn userdata (&self) -> Result<&Map<String, Value>> {
            if let Value::Object(projdata) = &self.userdata {
                Ok(projdata)
            } else {
                bail!("expeted user data to be a json object")
            }
        }
        fn partdata (&self) -> Result<&Map<String, Value>> {
            if let Value::Object(projdata) = &self.userdata {
                Ok(projdata)
            } else {
                bail!("expeted part data to be a json object")
            }
        }
    }


    fn is_html(p: &PathBuf) -> bool {
        if let Some(extension) = p.extension() {
            extension == "html"
        } else { false }
    }

}
