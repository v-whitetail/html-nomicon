#![allow(unused, dead_code)]
#![feature(pattern)]

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
        str::pattern::Pattern,
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
            let nodes = dfs(
                &self.template_dom.document,
                |n: &Rc<Node>| as_text_node_with_any(n, b.global_keys().iter())
                );
            globals.iter().for_each(
                |(key, value)| { nodes.iter().for_each(
                        |node| {
                            let text = as_text(node).expect("");
                            let sub = text.borrow().replace(&**key, &**value);
                            *text.borrow_mut() = StrTendril::from_slice(&sub);
                        });
                });
        }
    }


    fn dfs<P>(node: &Rc<Node>, predicate: P) -> Vec<Rc<Node>>
        where P: Fn(&Rc<Node>) -> Option<&Rc<Node>> + Copy
    {
        let mut set = node.children
            .borrow()
            .iter()
            .map( |child| dfs(child, predicate) )
            .flatten()
            .collect::<Vec<_>>();
        if let Some(data) = predicate(node) { set.push(data.clone()); }
        return set;
    }

    type Content = RefCell<StrTendril>;
    fn as_text_node(node: &Rc<Node>) -> Option<&Rc<Node>> {
        if let NodeData::Text { contents } = &node.data {
            Some( node )
        } else { None }
    }

    fn as_text(node: &Rc<Node>) -> Option<&Content> {
        if let NodeData::Text { contents } = &node.data {
            Some( contents )
        } else { None }
    }

    fn as_text_node_with<'a>(node: &'a Rc<Node>, tag: &Arc<str>)-> Option<&'a Rc<Node>> {
        if let Some(text) = as_text(node) {
            if text.borrow().contains(&**tag) {
                Some(node)
            } else { None }
        } else { None }
    }

    fn as_text_node_with_any<'a, T>(node: &'a Rc<Node>, tags: T) -> Option<&'a Rc<Node>>
        where T: Iterator<Item=&'a Arc<str>>
        {
            for tag in tags {
                if let Some(text) = as_text(node) {
                    if text.borrow().contains(&**tag) { return Some(node) };
                };
            };
            return None
        }

    type Attributes = RefCell<Vec<Attribute>>;
    fn as_attrs_node(node: &Rc<Node>) -> Option<&Rc<Node>> {
        if let NodeData::Element { attrs, .. } = &node.data {
            Some( node )
        } else { None }
    }

    fn as_attrs(node: &Rc<Node>) -> Option<&Attributes> {
        if let NodeData::Element { attrs, .. } = &node.data {
            Some( &attrs )
        } else { None }
    }

    fn as_attr_node_with<'a>(node: &'a Rc<Node>, tag: Arc<str>) -> Option<&'a Rc<Node>> {
        if let Some(attrs) = as_attrs(node) {
            for attr in attrs.borrow().iter() {
                if attr.value.contains(&*tag) { return Some(node) };
            };
        };
        return None
    }

    fn as_attr_node_with_any<'a, T>(node: &'a Rc<Node>, tags: T)-> Option<&'a Rc<Node>>
        where T: Iterator<Item=Arc<str>>
        {
            for tag in tags {
                if let Some(attrs) = as_attrs(node) {
                    for attr in attrs.borrow().iter() {
                        if attr.value.contains(&*tag) { return Some(node) };
                    };
                };
            };
            return None
        }
}
