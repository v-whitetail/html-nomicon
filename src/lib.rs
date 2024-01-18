#![allow(unused, dead_code)]
#![feature(string_remove_matches)]

pub mod cli;
pub mod buffer;
pub mod processing {

    use crate::{ cli::*, buffer::*, };
    use nom::IResult;
    use rayon::prelude::*;
    use anyhow::{ Result, bail, anyhow, };
    use std::{
        rc::Rc,
        cmp::max,
        sync::Arc,
        path::PathBuf,
        fs::{ write, read_dir, read_to_string, },
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
    }

}
