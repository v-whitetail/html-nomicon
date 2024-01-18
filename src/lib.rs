#![allow(unused, dead_code)]
#![feature(string_remove_matches)]

pub mod cli;
pub mod buffer;
pub mod processing {

    use crate::{ cli::*, buffer::*, };
    use rayon::prelude::*;
    use anyhow::{ Result, Error, bail, anyhow, };
    use std::{
        rc::Rc,
        cmp::max,
        sync::Arc,
        default::Default,
        io::{ self, Write, },
        fs::{ write, read_dir, },
        path::{ Path, PathBuf, },
    };
    use html5ever::{
        driver::ParseOpts,
        tendril::TendrilSink,
        tree_builder::TreeBuilderOpts,
        parse_document, serialize,
    };
    use markup5ever_rcdom::RcDom as Dom;


    #[derive(Debug, Clone)]
    pub struct Documents {
        pub root: PathBuf,
        pub reports: Box<[PathBuf]>,
        pub templates: Box<[PathBuf]>,
        pub resources: Box<[PathBuf]>,
    }
    impl Documents {
        pub fn new(path: &PathBuf) -> Result<Self> {
            let collect_dir = | r: &PathBuf, d: &str |
                Ok::<Box<[PathBuf]>, Error>(
                    read_dir(r.join(d))?
                    .filter_map( |entry| entry.ok() )
                    .map( |entry| entry.path() )
                    .collect::<Box<[_]>>()
                    );
            let root = path.canonicalize()?;
            let reports = collect_dir(&root, "Reports")?;
            let templates = collect_dir(&root, "Templates")?;
            let resources = collect_dir(&root, "Resources")?;
            Ok(Self{root, templates, reports, resources})
        }
    }

    fn parse_html<P>(p: P) -> Result<Dom> where P: AsRef<Path> {
        let opts = ParseOpts {
            tree_builder: TreeBuilderOpts {
                drop_doctype: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let dom = parse_document(Dom::default(), opts)
            .from_utf8()
            .from_file(p)?;
        Ok(dom)
    }

}
