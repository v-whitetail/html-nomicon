#![allow(unused, dead_code)]
#![feature(slice_group_by)]

pub mod cli;
pub mod buffer;
pub mod nomming;
pub mod processing {

    use crate::{ nomming::*, buffer::*, };
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






    #[derive(Debug, Clone)]
    pub struct Report<'b> {
        buffer: Buffer,
        raw_data: String,
        report_path: &'b PathBuf,
        template_path: &'b PathBuf,
    }
    impl<'b> Report<'b> {
        pub fn new(
            buffer: &Buffer,
            template_path: &'b PathBuf,
            report_path: &'b PathBuf,
            ) -> Result<Self> {
            let buffer = buffer.clone();
            let raw_data = read_to_string(template_path)?;
            Ok(Self{buffer, raw_data, template_path, report_path})
        }
        pub fn process(self) -> Result<()> {
            if let Ok((_, template)) = Template::new(&self.raw_data) {
                let buffer = self.buffer.sort(template.sorting_variable)?;
                let title_block = buffer.userdata
                    .iter()
                    .chain(buffer.projdata.iter())
                    .fold(
                        template.title_block.to_owned(),
                        |mut block, (k, v)| {
                            block.replace(&**k, &*v)
                        });
                let data_block = if let Some(part_map) = buffer.part_map {
                    let t_sorting_row = template
                        .sorting_row
                        .ok_or_else(||anyhow!("sorting row not found"))?;
                    let t_sorting_variable = template
                        .sorting_variable
                        .ok_or_else(||anyhow!("sorting varialbe not found"))?;
                    let t_pattern_row = template
                        .sorting_row
                        .ok_or_else(||anyhow!("pattern row not found"))?;
                    let data_block = part_map
                        .iter()
                        .map( |(sort_value, part_ids)| {
                            let sorting_row = t_sorting_row
                                .replace(t_sorting_variable, sort_value);
                            let pattern_rows = part_ids
                                .iter()
                                .map( |part_id| {
                                    let headers = &buffer
                                        .partdata
                                        .headers;
                                    let part_data = buffer
                                        .partdata
                                        .parts
                                        .get(part_id)
                                        .expect("partid mismatch");
                                    let pattern_row = headers
                                        .iter()
                                        .zip(part_data.iter())
                                        .fold(
                                            t_pattern_row.to_owned(),
                                            |mut row, (k, v)| {
                                                if let Some(v) = v.as_name() {
                                                    row.replace(&**k, &*v)
                                                } else { row }
                                            });
                                    pattern_row
                                }).collect::<String>();
                            let data_block = template.data_block
                                .replace(t_sorting_row, &sorting_row)
                                .replace(t_pattern_row, &pattern_rows);
                            data_block
                        }).collect::<String>();
                    data_block
                } else {
                    let data_block = buffer.partdata.parts
                        .iter()
                        .map( |(part_id, values)| {
                            buffer.partdata.headers
                                .iter()
                                .zip(values.iter())
                                .fold(
                                    template.data_block.to_owned(),
                                    |mut block, (k, v)| {
                                        if let Some(v) = v.as_name() {
                                            block.replace(&**k, &*v)
                                        } else { block }
                                    })
                        }).collect::<String>();
                    data_block
                };
                let report = &self.raw_data
                    .replace(template.title_block, &title_block)
                    .replace(template.data_block, &data_block);
                write(self.report_path, &report)?;
                Ok(())
            }
            else {
                let log = Template::new(&self.raw_data);
                eprintln!("{log:#?}");
                Err(anyhow!("template failed to parse"))
            }
        }
    }
}
