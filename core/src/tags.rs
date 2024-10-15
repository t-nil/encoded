use itertools::Itertools as _;
use nom::{
    branch::alt,
    bytes::complete::{tag as nom_tag, take_until, take_until1, take_while, take_while1},
    character::{complete::char, is_alphabetic, is_alphanumeric},
    combinator::{eof, peek, recognize, rest},
    multi::{many1, many_m_n, separated_list1},
    sequence::{delimited, separated_pair},
    IResult, Parser,
};
use tracing::{debug, error, trace};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagValue {
    parent: String,
    child: Option<Box<TagValue>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    Value(TagValue),
    KV { key: TagValue, val: TagValue },
}

pub fn parse_tags<'b>() -> impl FnMut(&'b str) -> IResult<&'b str, Vec<Tag>> {
    let non_tag =
        || alt((take_until::<&str, &str, nom::error::Error<&str>>("["), rest)).map(|_| "");
    delimited(non_tag(), separated_list1(non_tag(), tag()), non_tag())
}

fn tag<'b>() -> impl FnMut(&'b str) -> IResult<&'b str, Tag> {
    delimited(char('['), tag_label(), char(']'))
}

impl TagValue {
    pub fn from_slice_of_parts(ts: &[&str]) -> Option<Self> {
        match ts {
            [] => None,
            [first, rest @ ..] => Some(TagValue {
                parent: first.trim().to_string(),
                child: Self::from_slice_of_parts(rest).map(Box::new),
            }),
        }
    }
}

fn tag_label<'b>() -> impl FnMut(&'b str) -> IResult<&'b str, Tag> {
    // for lifetime inference - not relevant to parsing logic
    /*fn constrain<F>(f: F) -> F
    where
        F: for<'a> Fn(&'a str) -> IResult<&'a str, &'a str>,
    {
        f
    }*/

    let is_tag_text = |c: char| {
        error!("is_tag_text({c})");
        c.try_into().map(|c| is_alphanumeric(c)).unwrap_or(false)
            || "0123456789 -&@…()~"
                .chars()
                .chain("_∕".chars())
                .any(|valid_c| valid_c == c)
    };
    let simple_as_str = || {
        take_while1(is_tag_text)
        /*constrain(|tag: &str| -> IResult<&str, &str> {
            match tag.find("___") {
                Some(pos) => dbg!(Ok((&tag[pos..], &tag[..pos]))),
                None => Ok(("", tag)),
            }
        })*/
    };

    let hierarchical = || {
        simple_as_str()
            .map(|input: &str| input.split('∕').flat_map(|s| s.split("___")).collect_vec())
            .map(|tag_parts| {
                TagValue::from_slice_of_parts(&tag_parts)
                    .expect("we take_while1, so this can't be empty")
            })
    };
    let kv = separated_pair(hierarchical(), char('='), hierarchical()).map(|(key, val)| {
        error!("kv(c)");
        Tag::KV { key, val }
    });

    alt((kv, hierarchical().map(|tag_value| Tag::Value(tag_value))))
}

#[allow(non_snake_case)]
#[cfg(test)]
mod test {
    use color_eyre::{eyre::Error, Result};
    use insta::{assert_compact_debug_snapshot, assert_debug_snapshot};

    use super::*;
    #[test]
    fn Tag__parse__simple() -> Result<()> {
        assert_compact_debug_snapshot!(tag_label()("foobar")?, @r###"("", Value(TagValue { parent: "foobar", child: None }))"###);

        assert_compact_debug_snapshot!(tag()("[Hogwarts Legacy]"), @r###"Ok(("", Value(TagValue { parent: "Hogwarts Legacy", child: None })))"###);
        assert_compact_debug_snapshot!(tag()("[other∕gaming]"), @r###"Ok(("", Value(TagValue { parent: "other", child: Some(TagValue { parent: "gaming", child: None }) })))"###);
        assert_compact_debug_snapshot!(tag()("[other___gaming]"), @r###"Ok(("", Value(TagValue { parent: "other", child: Some(TagValue { parent: "gaming", child: None }) })))"###);
        assert_compact_debug_snapshot!(tag()("[@Ramon]"), @r###"Ok(("", Value(TagValue { parent: "@Ramon", child: None })))"###);
        assert_compact_debug_snapshot!(tag()("[mode=hardcore]"), @r###"Ok(("", KV { key: TagValue { parent: "mode", child: None }, val: TagValue { parent: "hardcore", child: None } }))"###);
        assert_compact_debug_snapshot!(tag()("[many∕mode=much___hardcore]"), @r###"Ok(("", KV { key: TagValue { parent: "many", child: Some(TagValue { parent: "mode", child: None }) }, val: TagValue { parent: "much", child: Some(TagValue { parent: "hardcore", child: None }) } }))"###);

        assert_compact_debug_snapshot!(parse_tags()("2024-02-03 15-23-46 [Hogwarts Legacy] and [other] stuff"), @r###"Ok(("", [Value(TagValue { parent: "Hogwarts Legacy", child: None }), Value(TagValue { parent: "other", child: None })]))"###);
        assert_debug_snapshot!(parse_tags()("2024-02-03 15-23-46 [game = Hogwarts Legacy] and [other∕stuff=my___important∕tag] stuff"), @r###"
        Ok(
            (
                "",
                [
                    KV {
                        key: TagValue {
                            parent: "game",
                            child: None,
                        },
                        val: TagValue {
                            parent: "Hogwarts Legacy",
                            child: None,
                        },
                    },
                    KV {
                        key: TagValue {
                            parent: "other",
                            child: Some(
                                TagValue {
                                    parent: "stuff",
                                    child: None,
                                },
                            ),
                        },
                        val: TagValue {
                            parent: "my",
                            child: Some(
                                TagValue {
                                    parent: "important",
                                    child: Some(
                                        TagValue {
                                            parent: "tag",
                                            child: None,
                                        },
                                    ),
                                },
                            ),
                        },
                    },
                ],
            ),
        )
        "###);

        Ok(())
    }
}
