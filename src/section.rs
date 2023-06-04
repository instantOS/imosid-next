// use crate::comment;
use crate::comment::CommentType;
use crate::{
    comment::Specialcomment,
    hashable::{CompileResult, Hashable},
};
use sha256::digest;

#[derive(Clone)]
pub enum Section {
    Named(SectionData, NamedSectionData),
    /// anonymous sections are sections without marker comments
    /// e.g. parts not tracked by imosid
    Anonymous(SectionData),
}

#[derive(Clone)]
pub struct NamedSectionData {
    pub name: String,           // section name, None if anonymous
    pub source: Option<String>, // source to update section from
    pub hash: String,           // current hash of section
    pub targethash: String,     // hash section should have if unmodified
}

#[derive(Clone)]
pub struct SectionData {
    startline: u32, // line number section starts at in file
    content: String,
    endline: u32, // line number section ends at in file
}

impl Hashable for Section {
    /// set target hash to current hash
    /// marking the section as unmodified
    /// return false if nothing has changed

    fn compile(&mut self) -> CompileResult {
        match self {
            Section::Named(_, named_data) => {
                if named_data.targethash == named_data.hash {
                    CompileResult::Unchanged
                } else {
                    named_data.targethash = named_data.hash.clone();
                    CompileResult::Changed
                }
            }
            Section::Anonymous(_) => CompileResult::Unchanged,
        }
    }

    /// generate section hash
    /// and detect section status
    fn finalize(&mut self) {
        if let Section::Named(data, named_data) = self {
            named_data.hash = digest(data.content.as_str()).to_uppercase();
        }
    }
}

impl Section {
    pub fn new(
        start: u32,
        end: u32,
        name: String,
        source: Option<String>,
        targethash: String,
    ) -> Section {
        Section::Named(
            SectionData {
                startline: start,
                content: String::from(""),
                endline: end,
            },
            NamedSectionData {
                name,
                source,
                hash: String::from(""),
                targethash,
            },
        )
    }

    /// append string to content
    //maybe make this a trait?
    pub fn push_str(&mut self, line: &str) {
        match self {
            Section::Named(data, _) => data,
            Section::Anonymous(data) => data,
        }
        .content
        .push_str(line)
    }

    /// return entire section with formatted marker comments and content
    pub fn output(&self, commentsign: &str) -> String {
        match self {
            Section::Named(data, named_data) => {
                let mut outstr = String::new();
                outstr.push_str(&Specialcomment::new_string(
                    commentsign,
                    CommentType::SectionBegin,
                    name,
                    None,
                ));
                outstr.push_str(&Specialcomment::new_string(
                    commentsign,
                    CommentType::HashInfo,
                    name,
                    Some(&if let Some(targethash) = self.targethash.clone() {
                        targethash
                    } else {
                        self.hash.clone()
                    }),
                ));
                if let Some(source) = &self.source {
                    outstr.push_str(&format!("{}... {} source {}\n", commentsign, name, source));
                }
                //todo: section target
                outstr.push_str(&self.content);
                outstr.push_str(&format!("{}... {} end\n", commentsign, name));
                outstr
            }
            Section::Anonymous(data) => data.content.clone()
        }
        match &self.name {
            Some(name) => 
        }
    }
}
