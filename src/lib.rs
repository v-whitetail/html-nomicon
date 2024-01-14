#![allow(unused, dead_code)]
#![feature(slice_group_by)]

pub mod cli;
pub mod buffer;
pub mod nomming;
pub mod processing {

    use nom::IResult;
    use rayon::prelude::*;
    use anyhow::{ Result, bail, anyhow, };
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
        sorting_variable: Option<&'b str>,
    }
    impl<'b> Template<'b> {
        pub fn new(s: &'b str) -> IResult<&str, Self> {
            let (_, body) = body(s)?;
            let (_, (title_block, data_block)) = blocks(body)?;
            let (_, (sorting_row, pattern_row)) = rows(data_block)?;
            let sorting_variable = sorting_row
                .and_then( |row| sorting_variable(row).ok() )
                .and_then( |(_, variable)| Some(variable));
            Ok((s, Self{
                body,
                data_block,
                title_block,
                sorting_row,
                pattern_row,
                sorting_variable,
            }))
        }
    }




    #[derive(Debug)]
    pub enum TemplateData<'b> {
        Raw(Box<[(String, Buffer)]>),
        Parsed(Box<[(Template<'b>, Buffer)]>),
    }
    impl<'b> TemplateData<'b> {
        pub fn new(documents: &Documents, buffer: &'b Buffer) -> Result<Self> {
            let templates = buffer
                .list_all_reports()?
                .iter() 
                .filter_map( |stem| documents.check_template(stem) )
                .filter_map( |path| read_to_string(path).ok() )
                .map( |file| (file, buffer.clone()) )
                .collect();
            Ok(Self::Raw(templates))
        }
        pub fn parse(&'b self) -> Result<Self> {
            match self {
                Self::Raw(raw_templates) => {
                    let templates = raw_templates
                        .par_iter()
                        .filter_map(|(template, buffer)|
                                     Template::new(template)
                                     .ok()
                                     .and_then( |(_, template)|
                                                Some((template, buffer))
                                              )
                                   )
                        .filter_map( |(template, buffer)|
                                     buffer
                                     .clone()
                                     .sort(template.sorting_variable)
                                     .ok()
                                     .and_then( |buffer|
                                                Some((template, buffer))
                                              )
                                   )
                        .collect();
                    Ok(Self::Parsed(templates))
                },
                _ => bail!("attempted to re-parse {self:#?}"),
            }
        }
    }
}
