// use crate::comment;
use crate::comment::CommentType;
use crate::{
    comment::Specialcomment,
    hashable::{CompileResult, Hashable},
};
use sha256::digest;

#[derive(Clone)]
pub struct Section {
    pub startline: u32,         // line number section starts at in file
    pub name: Option<String>,   // section name, None if anonymous
    pub source: Option<String>, // source to update section from
    pub endline: u32,           // line number section ends at in file
    pub hash: String,           // current hash of section
    targethash: Option<String>, // hash section should have if unmodified
    pub content: String,
    pub modified: bool,
}

impl Hashable for Section {
    /// set target hash to current hash
    /// marking the section as unmodified
    /// return false if nothing has changed
    fn compile(&mut self) -> CompileResult {
        let changed = match &self.targethash {
            Some(hash) => {
                if hash.to_string() == self.hash {
                    CompileResult::Unchanged
                } else {
                    CompileResult::Changed
                }
            }
            None => {
                if self.is_anonymous() {
                    CompileResult::Unchanged
                } else {
                    CompileResult::Changed
                }
            }
        };
        self.targethash = Some(self.hash.clone());
        changed
    }

    /// generate section hash
    /// and detect section status
    fn finalize(&mut self) {
        let newhash = digest(self.content.as_str()).to_uppercase();
        if self.name.is_some() {
            self.modified = self.hash != newhash;
        }
        self.hash = newhash;
    }
}

impl Section {
    pub fn new(
        start: u32,
        end: u32,
        name: Option<String>,
        source: Option<String>,
        targethash: Option<String>,
    ) -> Section {
        Section {
            name,
            startline: start,
            endline: end,
            source,
            hash: match &targethash {
                Some(hash) => String::from(hash),
                None => String::new(),
            },
            targethash,
            modified: false,
            content: String::from(""),
        }
    }

    /// anonymous sections are sections without marker comments
    /// e.g. parts not tracked by imosid
    pub fn is_anonymous(&self) -> bool {
        self.name.is_none()
    }

    /// append string to content
    //maybe make this a trait?
    pub fn push_str(&mut self, line: &str) {
        self.content.push_str(&format!("{line}\n"));
    }

    /// return entire section with formatted marker comments and content
    pub fn output(&self, commentsign: &str) -> String {
        match &self.name {
            Some(name) => {
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
            // anonymous section
            None => self.content.clone(),
        }
    }
}
