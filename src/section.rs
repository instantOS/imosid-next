// use crate::comment;
use crate::hashable::Hashable;
use sha256::digest;

#[derive(Clone)]
pub struct Section {
    pub startline: u32, // line number section starts at in file
    pub name: Option<String>, // section name, None if anonymous
    pub source: Option<String>, // source to update section from
    pub endline: u32, // line number section ends at in file
    pub hash: String, // current hash of section
    targethash: Option<String>, // hash section should have if unmodified
    pub content: String, 
    pub modified: bool,
}

impl Hashable for Section {
    /// set target hash to current hash
    /// marking the section as unmodified
    /// return false if nothing has changed
    fn compile(&mut self) -> bool {
        match &self.targethash {
            Some(hash) => {
                if hash.to_string() == self.hash {
                    return false;
                }
            }
            None => {
                return !self.is_anonymous();
            }
        }
        self.targethash = Option::Some(self.hash.clone());
        true
    }

    /// generate section hash
    /// and detect section status
    fn finalize(&mut self) {
        let newhash = digest(self.content.as_str()).to_uppercase();
        match &self.name {
            Some(_) => {
                if self.hash == newhash {
                    self.modified = false;
                } else {
                    self.modified = true;
                }
            }
            // anonymous section
            None => {
                self.hash = newhash.clone();
            }
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
        return self.name.is_none();
    }

    /// append string to content
    //maybe make this a trait?
    pub fn push_str(&mut self, line: &str) {
        self.content.push_str(&format!("{line}\n"));
    }

    /// return entire section with formatted marker comments and content
    pub fn output(&self, commentsign: &str) -> String {
        let mut outstr = String::new();
        match &self.name {
            Some(name) => {
                outstr.push_str(&format!("{}... {} begin\n", commentsign, name));
                outstr.push_str(&format!(
                    "{}... {} hash {}\n",
                    commentsign,
                    name,
                    if self.targethash.is_some() {
                        self.targethash.clone().unwrap()
                    } else {
                        self.hash.clone()
                    }
                ));
                match &self.source {
                    Some(source) => {
                        outstr.push_str(&format!("{}... {} begin\n", commentsign, source));
                    }
                    None => {}
                } //todo: section target
                outstr.push_str(&self.content);
                outstr.push_str(&format!("{}... {} end\n", commentsign, name));
            }
            // anonymous section
            None => {
                outstr = self.content.clone();
                return outstr;
            }
        }
        return outstr;
    }
}
