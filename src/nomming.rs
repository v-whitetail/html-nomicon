#![allow(unused, dead_code)]

use std::borrow::Cow;
use nom::{
    IResult,
    combinator::recognize,
    sequence::{ pair, tuple, preceded, delimited, },
    bytes::complete::{ tag, take_until, take_till, },
    character::complete::space0,
};

type PResult<'s> = IResult<&'s str, &'s str>;
fn variable<'s>(t:&'s str) -> impl Fn(&'s str) -> PResult<'s> {
    move |s| recognize(pair(tag("_"), tag(t)))(s)
}
fn take_once<'s>(t:&'s str) -> impl Fn(&'s str) -> PResult<'s> {
    move |s| recognize(pair(take_until(t), tag(t)))(s)
}
fn open_tag<'s>(t:&'s str) -> impl Fn(&'s str) -> PResult<'s> {
    move |s| recognize(delimited(take_once("<"), tag(t), space0))(s)
}
fn close_tag<'s>(t:&'s str) -> impl Fn(&'s str) -> PResult<'s> {
    move |s| recognize(delimited(take_once("</"), tag(t), tag(">")))(s)
}
fn class_name<'s>(t:&'s str) -> impl Fn(&'s str) -> PResult<'s> {
    move |s| recognize(pair(tag("class="), take_once(t)))(s)
}
fn class_tendril<'s>(t:&'s str, n:&'s str) -> impl Fn(&'s str) -> PResult<'s> {
    move |s| recognize(delimited(open_tag(t), class_name(n), close_tag(t)))(s)
}
fn noclass_tendril<'s>(t:&'s str) -> impl Fn(&'s str) -> PResult<'s> {
    move |s| recognize(pair(open_tag(t), close_tag(t)))(s)
}

//pub fn body(s: &str) -> IResult<&str, &str> {
//    Class::Body.parse(s)
//}
//pub fn sorting_row(s: &str) -> IResult<&str, &str> {
//    Class::Tr("sorting_row").parse(s)
//}
//pub fn pattern_row(s: &str) -> IResult<&str, &str> {
//    Class::Tr("pattern_row").parse(s)
//}
//pub fn title_block(s: &str) -> IResult<&str, &str> {
//    Class::Table("title_block").parse(s)
//}
//pub fn data_block(s: &str) -> IResult<&str, &str> {
//    Class::Table("data_block").parse(s)
//}
//pub fn rows(s: &str) -> IResult<&str, (&str, &str)> {
//    pair(sorting_row, pattern_row)(s)
//}
//pub fn blocks(s: &str) -> IResult<&str, (&str, &str)> {
//    pair(title_block, data_block)(s)
//}
