use crate::comment;

#[derive(Clone)]
pub struct Section {
    startline: u32,
    name: Option<String>,
    source: Option<String>,
    endline: u32,
    hash: String,
    targethash: Option<String>,
    content: String,
    modified: bool,
}

trait Hashable {
    fn finalize(&mut self);
    fn compile(&mut self) -> bool;
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
                if self.is_anonymous() {
                    return false;
                } else {
                    return true;
                }
            }
        }
        self.targethash = Option::Some(self.hash.clone());
        true
    }

    /// generate section hash
    /// and detect section status
    fn finalize(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(&self.content);
        let hasher = hasher.finalize();
        let newhash = format!("{:X}", hasher);
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
                self.hash = newhash;
            }
        }
        self.hash = String::from(format!("{:X}", hasher));
    }
}

impl Section {
    fn new(
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
    fn is_anonymous(&self) -> bool {
        match &self.name {
            Some(_) => false,
            None => true,
        }
    }

    /// append string to content
    //maybe make this a trait?
    fn push_str(&mut self, line: &str) {
        self.content.push_str(line);
        self.content.push('\n');
    }

    /// return entire section with formatted marker comments and content
    fn output(&self, commentsign: &str) -> String {
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
