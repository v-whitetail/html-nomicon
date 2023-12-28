use nom::{
    IResult,
    multi::separated_list0,
    branch::alt,
    sequence::{ pair, preceded, terminated, delimited, },
    combinator::{ map, opt, rest, peek, recognize, map_parser, iterator, },
    bytes::complete::{ tag, take_until, },
    character::complete::multispace0,
};


const TR: &'static str = "tr";
const TD: &'static str = "td";
const BODY: &'static str = "body";
const TABLE: &'static str = "table";
const DATA_BLOCK: &'static str = "data_block";
const TITLE_BLOCK: &'static str = "title_block";
const PATTERN_ROW: &'static str = "pattern_row";
const SORTING_ROW: &'static str = "sorting_row";
const OPEN: &'static str = "<";
const CLASS: &'static str = " class=\"";
const CLOSE: (&'static str, &'static str) = ("</",">");
const PREFIX: &'static str = "_";


pub fn body(s: &str) -> IResult<&str, &str> {
    element(BODY)(s)
}
pub fn data_block(s: &str) -> IResult<&str, &str> {
    class_element(TABLE, DATA_BLOCK)(s)
}
pub fn title_block(s: &str) -> IResult<&str, &str> {
    class_element(TABLE, TITLE_BLOCK)(s)
}
pub fn blocks(s: &str) -> IResult<&str, (&str, &str)> {
    pair(title_block, data_block)(s)
}
pub fn sorting_row(s: &str) -> IResult<&str, Option<&str>> {
    opt(class_element(TR, SORTING_ROW))(s)
}
pub fn pattern_row(s: &str) -> IResult<&str, Option<&str>> {
    opt(class_element(TR, PATTERN_ROW))(s)
}
pub fn rows(s: &str) -> IResult<&str, (Option<&str>, Option<&str>)> {
    pair(sorting_row, pattern_row)(s)
}


type Tag1<'s> = (&'s str);
type Tag2<'s> = (&'s str, &'s str);
type Tag3<'s> = (&'s str, &'s str, &'s str);
type Tag4<'s> = (&'s str, &'s str, &'s str, &'s str);
type FResult<'s> = IResult<&'s str, &'s str>;


fn tag2<'s>(t: Tag2<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| recognize(pair(tag(t.0), tag(t.1)))(s)
}
fn tag3<'s>(t: Tag3<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| recognize(pair(tag(t.0), tag2((t.1, t.2))))(s)
}
fn tag4<'s>(t: Tag4<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| recognize(pair(tag(t.0), tag3((t.1, t.2, t.3))))(s)
}
fn skip_to_tag<'s>(t: Tag1<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| preceded(take_until(t), peek(tag(t)))(s)
}
fn skip_to_tag2<'s>(t: Tag2<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| alt((
            preceded(take_until(t.0), peek(tag2(t))),
            preceded(take_with_tag(t.0), skip_to_tag2(t)),
            ))(s)
}
fn skip_to_tag3<'s>(t: Tag3<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| alt((
            preceded(take_until(t.0), peek(tag3(t))),
            preceded(take_with_tag(t.0), skip_to_tag3(t)),
            ))(s)
}
fn skip_to_tag4<'s>(t: Tag4<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| alt((
            preceded(take_until(t.0), peek(tag4(t))),
            preceded(take_with_tag(t.0), skip_to_tag4(t)),
            ))(s)
}
fn take_with_tag<'s>(t: Tag1<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| recognize(pair(skip_to_tag(t), tag(t)))(s)
}
fn take_with_tag2<'s>(t: Tag2<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| recognize(pair(skip_to_tag2(t), tag2(t)))(s)
}
fn take_with_tag3<'s>(t: Tag3<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| recognize(pair(skip_to_tag3(t), tag3(t)))(s)
}
fn take_with_tag4<'s>(t: Tag4<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| recognize(pair(skip_to_tag4(t), tag4(t)))(s)
}
fn trim_until_tag<'s>(t: Tag1<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| preceded(skip_to_tag(t), rest)(s)
}
fn trim_until_tag2<'s>(t: Tag2<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| preceded(skip_to_tag2(t), rest)(s)
}
fn trim_until_tag3<'s>(t: Tag3<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| preceded(skip_to_tag3(t), rest)(s)
}
fn trim_until_tag4<'s>(t: Tag4<'s>) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| preceded(skip_to_tag4(t), rest)(s)
}





fn open_element<'s>(e: &'s str) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| trim_until_tag2((OPEN, e))(s)
}
fn close_element<'s>(e: &'s str) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| recognize(take_with_tag3((CLOSE.0, e, CLOSE.1)))(s)
}





fn element<'s>(e:&'s str) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| map_parser(close_element(e), open_element(e))(s)
}
fn variable<'s>(v: &'s str) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| recognize(pair(tag(PREFIX), tag(v)))(s)
}
fn delimit_data_row() -> impl Fn(&'static str) -> FResult<'static> {
    move |s| take_with_tag3((CLOSE.0, TD, CLOSE.1))(s)
}
fn variable_cell<'s>(v: &'s str) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| preceded(take_until(PREFIX), variable(v))(s)
}
fn variable_element<'s>(v:&'s str) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| map_parser(element(TD), variable_cell(v))(s)
}
fn tag_class<'s>(e:&'s str, c:&'s str) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| map_parser(element(e), trim_until_tag4((OPEN, e, CLASS, c)))(s)
}
fn class_element<'s>(e:&'s str, c:&'s str) -> impl Fn(&'s str) -> FResult<'s> {
    move |s| alt((
            tag_class(e, c),
            preceded(element(e), class_element(e, c))
            ))(s)
}
