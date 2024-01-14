#![allow(unused, dead_code)]
#![feature(slice_group_by)]

pub mod cli;
pub mod buffer;
pub mod nomming;
pub mod processing {

    use nom::IResult;
    use rayon::prelude::*;
    use anyhow::{ Result, bail, };
    use crate::{ nomming::*, buffer::*, };
    use std::{
        cmp::max,
        path::PathBuf,
        fs::{ read_dir, read_to_string, },
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




    #[derive(Debug)]
    pub enum TemplateData<'b> {
        Raw(Box<[String]>),
        Parsed(Box<[(Template<'b>, Buffer)]>),
    }
    impl<'b, 'a: 'b> TemplateData<'b> {
        pub fn new(buffer: &'b Buffer, documents: &Documents) -> Result<Self> {
            let templates = buffer
                .list_all_reports()?
                .iter() 
                .filter_map( |stem| documents.check_template(stem) )
                .filter_map( |path| read_to_string(path).ok() )
                .collect();
            Ok(Self::Raw(templates))
        }
        pub fn few(&'a self, buffer: &'b Buffer) -> Result<Self> {
            match self {
                Self::Raw(raw_templates) => {
                    let templates = raw_templates
                        .par_iter()
                        .filter_map( |template| Template::new(template).ok() )
                        .map( |(input, output)| (output, buffer.clone()) )
                        .collect();
                    Ok(Self::Parsed(templates))
                },
                _ => bail!("attempted to re-parse {self:#?}"),
            }
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
    pub struct ParsedTemplates<'b> {
        templates: Box<[(Template<'b>, Buffer)]>,
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
                .map( |(input, output)| (output, buffer.clone()) )
                .collect();
            Ok(Self{templates})
        }
    }
}
