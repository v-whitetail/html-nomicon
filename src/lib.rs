#![allow(unused, dead_code)]

pub mod cli;

pub mod nomming {

    use std::{
        path::PathBuf,
        fs::{
            ReadDir,
            read_dir,
        },
    };
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
    use serde::{Serialize, Deserialize};
    use serde_json::{Map, Value};





    #[derive(Debug)]
    pub struct Documents {
        templates: ReadDir,
        reports: ReadDir,
        resources: ReadDir,
    }
    impl Documents {
        fn new(path: &PathBuf) -> Result<Self> {
            let templates = read_dir(path.join("Templates"))?;
            let reports = read_dir(path.join("Reports"))?;
            let resources = read_dir(path.join("resources"))?;
            Ok(Self{templates, reports, resources})
        }
    }





    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Buffer {
        projdata: Map<String, Value>,
        userdata: Map<String, Value>,
        partdata: Map<String, Value>,
    }





//    #[derive(Debug, Clone)]
//    pub struct Template<'t> {
//        body: &'t str,
//        title_block: &'t str,
//        data_block: &'t str,
//        sort_by_row: &'t str,
//        pattern_row: &'t str,
//    }
//    impl<'t> Template<'t> {
//        pub fn new(s: &'t str) -> IResult<&str, Self> {
//            let (_, body) = Self::body(s)?;
//            let (_, (title_block, data_block)) = Self::blocks(s)?;
//            let (_, (sort_by_row, pattern_row)) = Self::rows(s)?;
//            Ok((s, Self{
//                body,
//                title_block,
//                data_block,
//                sort_by_row,
//                pattern_row,
//            }))
//        }
//        fn blocks(s: &str) -> IResult<&str, (&str, &str)> {
//            tuple((
//                    Self::title_block,
//                    Self::data_block,
//                    ))(s)
//        }
//        fn rows(s: &str) -> IResult<&str, (&str, &str)> {
//            tuple((
//                    Self::sort_by_row,
//                    Self::pattern_row,
//                    ))(s)
//        }
//        fn body(s: &str) -> IResult<&str, &str> {
//            preceded(
//                pair(
//                    take_until("<body>"),
//                    tag("<body>")
//                    ),
//                    take_until("</body>")
//                    )(s)
//        }
//        fn title_block(s: &str) -> IResult<&str, &str> {
//            preceded(
//                pair(
//                    take_until("<table class=\"title_block\">"),
//                    tag("<table class=\"title_block\">")
//                    ),
//                    take_until("</table>")
//                    )(s)
//        }
//        fn data_block(s: &str) -> IResult<&str, &str> {
//            preceded(
//                pair(
//                    take_until("<table class=\"data_block\">"),
//                    tag("<table class=\"data_block\">")
//                    ),
//                    take_until("</table>")
//                    )(s)
//        }
//        fn sort_by_row(s: &str) -> IResult<&str, &str> {
//            preceded(
//                pair(
//                    take_until("<tr class=\"sort_by_row\">"),
//                    tag("<tr class=\"sort_by_row\">")
//                    ),
//                    take_until("</tr>")
//                    )(s)
//        }
//        fn pattern_row(s: &str) -> IResult<&str, &str> {
//            preceded(
//                pair(
//                    take_until("<tr class=\"pattern_row\">"),
//                    tag("<tr class=\"pattern_row\">")
//                    ),
//                    take_until("</tr>")
//                    )(s)
//        }
//    }
//
//
//
//    pub struct BatchProcessor {
//        buffer: Buffer,
//        documents: Documents,
//    }
//    impl<'b> BatchProcessor {
//        pub fn new(json: &'b Map<String, Value>, path: &PathBuf) -> Result<Self> {
//
//            let buffer = Buffer::new(json)?;
//            let documents = Documents::new(path)?;
//            buffer.listed_reports()?;
//
//            Ok(Self{buffer, documents})
//        }
//    }
}

