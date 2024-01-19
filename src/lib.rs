#![allow(unused, dead_code)]
#![feature(iterator_try_collect)]

pub mod cli;
pub mod buffer;
pub mod processing {

    use crate::{ cli::*, buffer::*, };
    use rayon::prelude::*;
    use anyhow::{ Result, Error, bail, anyhow, };
    use std::{
        rc::Rc,
        cmp::max,
        ops::Deref,
        sync::Arc,
        cell::RefCell,
        default::Default,
        io::{ self, Write, },
        fs::{ write, read_dir, },
        path::{ Path, PathBuf, },
    };
    use html5ever::{
        serialize,
        parse_document,
        Attribute,
        driver::ParseOpts,
        tree_builder::TreeBuilderOpts,
        tendril::{ TendrilSink, Tendril, StrTendril, },
    };
    use markup5ever_rcdom::{
        Node, NodeData,
        RcDom as Dom,
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
            let collect_dir = | r: &PathBuf, d: &str |
                Ok::<Box<[PathBuf]>, Error>
                (
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

    pub struct Report<'p> {
        template_path: &'p PathBuf,
        template_dom: Dom,
    }
    impl<'p> Report<'p> {
        pub fn new(template_path: &'p PathBuf) -> Result<Self> {
            let opts = ParseOpts {
                tree_builder: TreeBuilderOpts {
                    drop_doctype: true,
                    ..Default::default()
                },
                ..Default::default()
            };
            let template_dom = parse_document(Dom::default(), opts)
                .from_utf8()
                .from_file(template_path)?;
            Ok(Self{template_path, template_dom}) 
        }
        pub fn replace_globals(&self, b: &Buffer) {
            let globals = b.globals();
            let keys = globals.iter().map( |(key, _)| &**key);
            let nodes = str_dfs(
                &self.template_dom.document,
                |n: &Rc<Node>| multi_str_recognize(n, keys.clone())
                );
            globals.iter().for_each(
                |(key, value)| { nodes.iter().for_each(
                        |node| {
                            let sub = node.borrow().replace(&**key, &**value);
                            *node.borrow_mut() = StrTendril::from_slice(&sub);
                        });
                });
        }
    }

    type Content = RefCell<StrTendril>;
    type Attributes = RefCell<Vec<Attribute>>;

    fn dfs<P>(node: &Rc<Node>, predicate: P) -> Vec<Node>
        where P: Fn(&Rc<Node>) -> Option<Node> + Copy
    {
        let mut set = node.children
            .borrow()
            .iter()
            .map( |child| dfs(child, predicate) )
            .flatten()
            .collect::<Vec<_>>();
        if let Some(data) = predicate(node) { set.push(data); }
        return set;
    }

    fn str_dfs<P>(node: &Rc<Node>, predicate: P) -> Vec<Content>
        where P: Fn(&Rc<Node>) -> Option<Content> + Copy
    {
        let mut set = node.children
            .borrow()
            .iter()
            .map( |child| str_dfs(child, predicate) )
            .flatten()
            .collect::<Vec<_>>();
        if let Some(data) = predicate(node) { set.push(data); }
        return set;
    }

    fn str_tendril(node: &Rc<Node>) -> Option<Content> {
        if let NodeData::Text { contents } = &node.data {
            Some( contents.clone() )
        } else { None }
    }

    fn str_recognize(node: &Rc<Node>, tag: &str) -> Option<Content> {
        if let Some(content) = str_tendril(node) {
            content.borrow().contains(tag).then_some( content.clone() )
        } else { None }
    }

    fn multi_str_recognize<'t, I>(node: &Rc<Node>, tags: I) -> Option<Content>
        where I: IntoIterator<Item=&'t str>
    {
        if let Some(content) = str_tendril(node) {
            tags.into_iter()
                .filter( |tag| content.borrow().contains(tag))
                .fuse()
                .next()
                .and_then(|_| Some(content.clone()) )
        } else { None }
    }

    fn attr_tendril(node: &Rc<Node>) -> Option<Attributes> {
        if let NodeData::Element { attrs, .. } = &node.data {
            Some( attrs.clone() )
        } else { None }
    }

//    fn attr_recognize(node: &Rc<Node>, tag: &str) -> Option<Content> {
//        if let Some(attributes) = attr_tendril(node) {
//            attributes.borrow().iter().filter_map(
//                |attr| attr.value.contains(tag).then_some( attr.value.clone() )
//                )
//                .fuse()
//                .next()
//        } else { None }
//    }

}
