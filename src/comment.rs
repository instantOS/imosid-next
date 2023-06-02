use regex::Regex;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CommentType {
    SectionBegin,
    SectionEnd,
    SourceInfo,
    TargetInfo,
    HashInfo,
    PermissionInfo,
}

#[derive(Clone)]
pub struct Specialcomment {
    pub line: u32, // line number comment is at in file
    content: String,
    pub section: String, // section name extracted from prefix
    pub ctype: CommentType,
    pub argument: Option<String>, // optional argument, used for hashes etc
}

impl Specialcomment {
    pub fn type_from_keyword(keyword: &str) -> Option<CommentType> {
        Some(match keyword {
            "begin" | "start" => CommentType::SectionBegin,
            "end" | "stop" => CommentType::SectionEnd,
            "hash" => CommentType::HashInfo,
            "source" => CommentType::SourceInfo,
            "permissions" => CommentType::PermissionInfo,
            "target" => CommentType::TargetInfo,
            &_ => {
                return Option::None;
            }
        })
    }

    pub fn new(line: &str, commentsymbol: &str, linenumber: u32) -> Option<Specialcomment> {
        if !line.starts_with(commentsymbol) {
            return Option::None;
        }

        // construct regex that matches valid comments
        let mut iscomment = String::from("^ *");
        iscomment.push_str(&commentsymbol);
        iscomment.push_str(" *\\.\\.\\. *(.*)");
        let commentregex = Regex::new(&iscomment).unwrap();

        let keywords = commentregex.captures(&line);

        if let Some(captures) = &keywords {
            let keywords = captures
                .get(1)
                .unwrap()
                .as_str()
                .split(" ")
                .collect::<Vec<&str>>();

            // needs at least a section and a keyword
            if keywords.len() < 2 {
                return Option::None;
            }

            let sectionname = keywords[0];
            let keyword = keywords[1];
            //comment argument, example #...all source ARGUMENT
            let cargument: Option<String> = if keywords.len() > 2 {
                Option::Some(String::from(keywords[2]))
            } else {
                Option::None
            };

            let tmptype: CommentType;
            tmptype = Specialcomment::type_from_keyword(keyword)?;
            match tmptype {
                CommentType::HashInfo => {
                    if cargument == None {
                        println!("missing hash value on line {}", linenumber);
                        return Option::None;
                    }
                }
                CommentType::SourceInfo => {
                    match cargument {
                        Some(_) => {
                            println!("updating from source not implemented yet");
                            unimplemented!();
                            //TODO do something
                            //fetch from file/url/git
                        }
                        None => {
                            println!("missing source file on line {}", linenumber);
                            return Option::None;
                        }
                    }
                }
                CommentType::PermissionInfo => {
                    // permissioms can only be set for the entire file
                    if sectionname != "all" {
                        return Option::None;
                    }
                    match &cargument {
                        None => {
                            return Option::None;
                        }
                        //todo: more validation. maybe own permission type?
                        Some(arg) => match arg.parse::<u32>() {
                            Err(_) => {
                                return Option::None;
                            }
                            Ok(_) => {}
                        },
                    }
                }
                CommentType::TargetInfo => {
                    if sectionname == "all" {
                        if cargument == None {
                            println!("missing target value on line {}", linenumber);
                            return Option::None;
                        }
                    } else {
                        println!(
                            "warning: target can only apply to the whole file {}",
                            linenumber
                        );
                        return Option::None;
                    }
                }
                _ => {
                    println!("warning: incomplete imosid comment on {}", linenumber);
                    return Option::None;
                }
            }

            return Some(Specialcomment {
                line: linenumber,
                content: String::from(line),
                section: String::from(sectionname),
                ctype: tmptype,
                argument: cargument,
            });
        };
        return Option::None;
    }
}
