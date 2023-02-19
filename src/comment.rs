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
    pub line: u32,
    content: String,
    pub section: String,
    pub ctype: CommentType,
    pub argument: Option<String>,
}

impl Specialcomment {
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

        match &keywords {
            Some(captures) => {
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
                let cargument: Option<String>;

                if keywords.len() > 2 {
                    cargument = Option::Some(String::from(keywords[2]));
                } else {
                    cargument = Option::None;
                }

                let tmptype: CommentType;
                match keyword {
                    "begin" | "start" => {
                        tmptype = CommentType::SectionBegin;
                    }
                    "end" | "stop" => {
                        tmptype = CommentType::SectionEnd;
                    }
                    "hash" => {
                        tmptype = CommentType::HashInfo;
                        match cargument {
                            Some(_) => {}
                            None => {
                                println!("missing hash value on line {}", linenumber);
                                return Option::None;
                            }
                        }
                    }
                    "source" => {
                        tmptype = CommentType::SourceInfo;
                        match cargument {
                            Some(_) => {
                                println!("updating from source not implemented yet");
                                //TODO do something
                                //fetch from file/url/git
                            }
                            None => {
                                println!("missing source file on line {}", linenumber);
                                return Option::None;
                            }
                        }
                    }
                    "permissions" => {
                        // permissioms can only be set for the entire file
                        if sectionname != "all" {
                            return Option::None;
                        }
                        match &cargument {
                            None => {
                                return Option::None;
                            }
                            Some(arg) => match arg.parse::<u32>() {
                                Err(_) => {
                                    return Option::None;
                                }
                                Ok(_) => {
                                    tmptype = CommentType::PermissionInfo;
                                }
                            },
                        }
                    }
                    "target" => {
                        if sectionname == "all" {
                            tmptype = CommentType::TargetInfo;
                            match cargument {
                                Some(_) => {}
                                None => {
                                    println!("missing target value on line {}", linenumber);
                                    return Option::None;
                                }
                            }
                        } else {
                            println!(
                                "warning: target can only apply to the whole file {}",
                                linenumber
                            );
                            return Option::None;
                        }
                    }

                    &_ => {
                        println!("warning: incomplete imosid comment on {}", linenumber);
                        return Option::None;
                    }
                }

                Option::Some(Specialcomment {
                    line: linenumber,
                    content: String::from(line),
                    section: String::from(sectionname),
                    ctype: tmptype,
                    argument: cargument,
                })
            }
            None => {
                return Option::None;
            }
        }
    }
}
