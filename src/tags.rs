use nom::{
    branch::alt,
    bytes::complete::take_while,
    multi::{many1, separated_list1},
    sequence::{delimited, separated_pair},
    IResult,
};

pub struct TagValue {
    parent: String,
    child: Option<Box<Tag>>,
}

pub enum Tag {
    Value(TagValue),
    KV { key: TagValue, val: TagValue },
}

pub fn parse_tags(s: &str) -> IResult<&str, Vec<Tag>> {
    many1(tag)(s)
}

fn tag(s: &str) -> IResult<&str, Tag> {
    delimited(char('('), tag_label, char(')'))
}

fn tag_label(s: &str) -> IResult<&str, Tag> {
    let is_tag_text = |c| is_alphabetic(c) || c == ' ';
    let simple = take_while(is_tag_text);

    let hierarchical = separated_list1(char('/'), simple);

    let kv = separated_pair(hierarchical, char('='), hierarchical);

    // simple is a special case of hierarchical, hence handled
    let result = hierarchical(s)?;
    if result.0 == "" {
        return Tag::Value(TagValue {
            parent: result.1.to_owned(),
            child: None,
        });
    }
    let result = kv(s);
    if result.0 == "" {}

    return;
}
