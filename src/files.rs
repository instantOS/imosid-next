use crate::built_info;
use crate::comment::{CommentType, Specialcomment};
use crate::contentline::ContentLine;
use crate::hashable::{Hashable, CompileResult};
use crate::section::Section;
use colored::Colorize;
use regex::Regex;
use semver::Version;
use sha256::digest;
use std::collections::HashMap;
use std::env::home_dir;
use std::ffi::OsStr;
use std::fs::{self, read_to_string, File, OpenOptions};
use std::io::{self, prelude::*, ErrorKind};
use std::ops::Deref;
use std::os::unix::prelude::PermissionsExt;
use std::path::{Path, PathBuf};
use toml::Value;

// a file containing metadata about an imosid file for file types which do not support comments
pub struct Metafile {
    pub hash: String,
    parentfile: String,
    targetfile: Option<String>,
    sourcefile: Option<String>,
    pub modified: bool,
    imosidversion: Version,
    syntaxversion: i64,
    value: Value,
    content: String,
    path: PathBuf,
    permissions: Option<u32>,
}

pub enum ApplyResult {
    Changed,
    Unchanged,
    Error,
}

impl Hashable for Metafile {
    // check for modifications
    fn finalize(&mut self) {
        self.modified = self.hash != self.get_content_hash();
    }

    fn compile(&mut self) -> CompileResult {
        let contenthash = self.get_content_hash();
        self.modified = false;
        if self.hash == contenthash {
            CompileResult::Unchanged
        } else {
            self.hash = contenthash;
            CompileResult::Changed
        }
    }
}

impl Metafile {
    fn new(path: PathBuf, content: &str) -> Option<Metafile> {
        if !path.is_file() {
            return None;
        }
        let metacontent = read_to_string(&path);
        match metacontent {
            Err(_) => {
                return None;
            }
            Ok(mcontent) => {
                let value = mcontent.parse::<Value>().expect("failed to read toml");

                let mut retfile = Metafile {
                    targetfile: None,
                    sourcefile: None,
                    hash: String::from(""),
                    parentfile: String::from(""),
                    // default version strings
                    imosidversion: Version::new(0, 0, 0),
                    syntaxversion: 1,
                    value: value.clone(),
                    content: String::from(content),
                    modified: false,
                    permissions: Option::None,
                    path,
                };

                // hash and parent are mandatory
                if let Some(Value::String(hash)) = value.get("hash") {
                    retfile.hash = String::from(hash);
                } else {
                    return None;
                }

                if let Some(Value::String(parentfile)) = value.get("parent") {
                    retfile.parentfile = String::from(parentfile);
                } else {
                    return None;
                }

                if let Some(Value::String(targetfile)) = value.get("target") {
                    retfile.targetfile = Some(String::from(targetfile));
                }

                if let Some(Value::String(sourcefile)) = value.get("source") {
                    retfile.sourcefile = Some(String::from(sourcefile));
                }

                if let Some(Value::Integer(permissions)) = value.get("permissions") {
                    //TODO check if permissions smaller than 777
                    retfile.permissions = Some(*permissions as u32);
                }

                if let Some(Value::Integer(syntaxversion)) = value.get("syntaxversion") {
                    retfile.syntaxversion = syntaxversion.clone();
                }

                if let Some(Value::String(imosidversion)) = value.get("imosidversion") {
                    if let Ok(version) = Version::parse(imosidversion) {
                        retfile.imosidversion = version;
                    }
                }

                return Some(retfile);
            }
        };
    }

    fn write_permissions(&self) {
        let mut parentpath = self.path.clone();
        parentpath.pop();
        parentpath.push(&self.parentfile);
        if let Some(permissions) = &self.permissions {
            let mut perms = fs::metadata(&parentpath).unwrap().permissions();
            let permint = u32::from_str_radix(&format!("{}", permissions + 1000000), 8).unwrap();
            perms.set_mode(permint);
            fs::set_permissions(&parentpath, perms).expect("failed to set permissions");
        } else {
            println!("no permissions");
        }
    }

    // create a new metafile for a file
    // TODO maybe return result?
    pub fn from(mut path: PathBuf) -> Metafile {
        //TODO handle result
        let filecontent =
            read_to_string(&path).expect("could not read file content to create metafile");

        let parentname = path
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();

        //TODO don't create metafiles for metafiles

        let filename = format!("{}.imosid.toml", parentname);

        path.pop();
        path.push(filename);

        let mut retfile: Metafile;
        //Maybe distinguish between new and from path?
        if path.is_file() {
            retfile = Metafile::new(path.clone(), &filecontent).expect("could not create metafile");
            retfile.update();
            retfile.finalize();
        } else {
            retfile = Metafile {
                targetfile: None,
                sourcefile: None,
                hash: String::from(""),
                parentfile: String::from(&parentname),
                imosidversion: Version::parse(built_info::PKG_VERSION).unwrap(),
                syntaxversion: 0,
                value: Value::Integer(0),
                content: String::from(&filecontent),
                modified: false,
                permissions: Option::None,
                path,
            };

            retfile.update();
            retfile.compile();
            retfile.write_to_file();
        }

        retfile
    }

    fn get_content_hash(&self) -> String {
        digest(self.content.clone()).to_uppercase()
    }

    // populate toml value with data
    fn update(&mut self) {
        let mut selfmap = toml::map::Map::new();
        selfmap.insert(
            String::from("hash"),
            Value::String(String::from(&self.hash)),
        );
        selfmap.insert(
            String::from("parent"),
            Value::String(String::from(&self.parentfile)),
        );
        if let Some(targetfile) = &self.targetfile {
            selfmap.insert(
                String::from("target"),
                Value::String(String::from(targetfile)),
            );
        }
        if let Some(sourcefile) = &self.sourcefile {
            selfmap.insert(
                String::from("source"),
                Value::String(String::from(sourcefile)),
            );
        }

        selfmap.insert(String::from("syntaxversion"), Value::Integer(0));

        selfmap.insert(
            String::from("imosidversion"),
            Value::String(self.imosidversion.to_string()),
        );

        selfmap.insert(
            String::from("syntaxversion"),
            Value::String(self.syntaxversion.to_string()),
        );
        self.value = Value::Table(selfmap);
    }

    pub fn output(&mut self) -> String {
        self.update();
        self.value.to_string()
    }

    pub fn write_to_file(&mut self) {
        let newfile = File::create(&self.path);
        match newfile {
            Err(_) => {
                eprintln!("{}", "Error: could not write metafile".red());
            }
            Ok(mut file) => {
                file.write_all(self.output().as_bytes())
                    .expect("could not write metafile");
            }
        }
    }
}

pub struct Specialfile {
    //TODO maybe implement finalize?
    specialcomments: Vec<Specialcomment>,
    pub sections: Vec<Section>,
    pub file: File,
    pub filename: String,
    pub targetfile: Option<String>,
    pub metafile: Option<Metafile>,
    pub commentsign: String,
    pub modified: bool,
    pub permissions: Option<u32>,
}

impl Specialfile {
    pub fn new(filename: &str) -> Result<Specialfile, std::io::Error> {
        let filepath = PathBuf::from(filename);
        Self::from_pathbuf(&filepath)
    }

    pub fn from_pathbuf(path: &PathBuf) -> Result<Specialfile, std::io::Error> {
        let sourcepath = path
            .canonicalize()
            .expect("could not canonicalize path")
            .display()
            .to_string();

        let sourcefile = match OpenOptions::new().read(true).write(true).open(path) {
            Err(e) => {
                if e.kind() == ErrorKind::PermissionDenied {
                    // open file as readonly if writing is not permitted
                    match OpenOptions::new().read(true).write(false).open(path) {
                        Ok(file) => file,
                        Err(error) => return Err(error),
                    }
                } else {
                    return Err(e);
                }
            }
            Ok(file) => file,
        };

        let metafile;

        let mut commentvector = Vec::new();
        let mut counter = 0;

        let mut sectionvector: Vec<Section> = Vec::new();
        let mut contentvector: Vec<ContentLine> = Vec::new();

        let mut sectionmap: HashMap<String, Vec<Specialcomment>> = HashMap::new();

        let mut targetfile: Option<String> = Option::None;
        let mut permissions = Option::None;
        let mut commentsign = String::new();
        let mut hascommentsign = false;

        // check for metafile
        if Path::new(&format!("{}.imosid.toml", sourcepath)).is_file() {
            let mut contentstring = String::new();
            io::BufReader::new(&sourcefile).read_to_string(&mut contentstring)?;

            metafile = if let Some(mut metafile) = Metafile::new(
                PathBuf::from(&format!("{}.imosid.toml", sourcepath)),
                &contentstring,
            ) {
                metafile.finalize();
                metafile
            } else {
                return Err(std::io::Error::new(ErrorKind::Other, "invalid metafile"));
            };
            return Ok(Specialfile {
                specialcomments: commentvector,
                sections: sectionvector,
                file: sourcefile,
                filename: sourcepath,
                targetfile: metafile.targetfile.clone(),
                modified: metafile.modified,
                permissions: metafile.permissions.clone(),
                metafile: Some(metafile),
                commentsign: String::from(""),
            });
        } else {
            let filelines = io::BufReader::new(&sourcefile).lines();

            // parse lines for special comments
            for i in filelines {
                counter += 1;
                let line = i?;
                if !hascommentsign {
                    commentsign = String::from(get_comment_sign(&sourcepath, &line));
                    hascommentsign = true;
                }
                let newcomment = Specialcomment::new(&line, &commentsign, counter);
                match newcomment {
                    Some(comment) => {
                        // comments with section all apply to the entire file
                        if &comment.section == "all" {
                            match &comment.ctype {
                                CommentType::TargetInfo => {
                                    if comment.argument.is_some() {
                                        targetfile =
                                            Option::Some(String::from(&comment.argument.unwrap()));
                                    }
                                }
                                CommentType::PermissionInfo => {
                                    if let Some(arg) = comment.argument {
                                        permissions = match arg.split_at(3).1.parse::<u32>() {
                                            Err(_) => Option::None,
                                            Ok(permnumber) => Option::Some(permnumber),
                                        }
                                    }
                                }
                                &_ => {}
                            }
                            continue;
                        }
                        commentvector.push(comment.clone());
                        if sectionmap.contains_key(&comment.section) {
                            sectionmap.get_mut(&comment.section).unwrap().push(comment);
                        } else {
                            let mut sectionvector = Vec::new();
                            sectionvector.push(comment.clone());
                            sectionmap.insert(comment.section, sectionvector);
                        }
                    }
                    None => contentvector.push(ContentLine {
                        linenumber: counter,
                        content: line,
                    }),
                }
            }

            // validate sections and initialze section structs
            for (sectionname, svector) in sectionmap.iter() {
                let mut checkmap = HashMap::new();
                // sections cannot have multiple hashes, beginnings etc
                for i in svector.iter() {
                    if checkmap.contains_key(&i.ctype) {
                        break;
                    } else {
                        checkmap.insert(&i.ctype, i);
                    }
                }
                if !(checkmap.contains_key(&CommentType::SectionBegin)
                    && checkmap.contains_key(&CommentType::SectionEnd)
                    && checkmap.contains_key(&CommentType::HashInfo))
                {
                    println!("warning: invalid section {}", sectionname);
                    continue;
                }

                let newsection = Section::new(
                    checkmap.get(&CommentType::SectionBegin).unwrap().line,
                    checkmap.get(&CommentType::SectionEnd).unwrap().line,
                    Option::Some(String::from(sectionname)),
                    match checkmap.get(&CommentType::SourceInfo) {
                        Some(source) => Some(String::from(source.argument.clone().unwrap())),
                        None => None,
                    },
                    Option::Some(
                        checkmap
                            .get(&CommentType::HashInfo)
                            .unwrap()
                            .argument
                            .clone()
                            .unwrap()
                            .clone(),
                    ),
                );

                sectionvector.push(newsection);
            }

            // sort sections by lines (retaining the original order of the file)
            sectionvector.sort_by(|a, b| a.startline.cmp(&b.startline));

            // detect overlapping sections
            let vecsize = sectionvector.len();
            let mut broken_indices = Vec::new();
            let mut skipnext = false;
            for i in 0..vecsize {
                if skipnext {
                    skipnext = false;
                    continue;
                }
                let currentsection = &sectionvector[i];
                if i < vecsize - 1 {
                    let nextsection = &sectionvector[i + 1];
                    if nextsection.startline < currentsection.endline {
                        broken_indices.push(i + 1);
                        broken_indices.push(i);
                        skipnext = true;
                    }
                }
            }

            for i in broken_indices {
                println!("section {} overlapping", i);
                sectionvector.remove(i);
            }

            let mut modified = false;
            // introduce anonymous sections
            if sectionvector.len() > 0 {
                let mut currentline = 1;
                let mut tmpstart;
                let mut tmpend;
                let mut anonvector: Vec<Section> = Vec::new();
                for i in &sectionvector {
                    if i.startline - currentline >= 1 {
                        tmpstart = currentline;
                        tmpend = i.startline - 1;
                        let newsection = Section::new(
                            tmpstart,
                            tmpend,
                            Option::None,
                            Option::None,
                            Option::None,
                        );
                        anonvector.push(newsection);
                    }
                    currentline = i.endline + 1;
                }

                sectionvector.extend(anonvector);
                sectionvector.sort_by(|a, b| a.startline.cmp(&b.startline));
            } else {
                let newsection = Section::new(
                    1,
                    contentvector.len() as u32,
                    Option::None,
                    Option::None,
                    Option::None,
                );
                sectionvector.push(newsection);
            }

            // fill sections with content
            for i in &mut sectionvector {
                // TODO: speed this up, binary search or something
                for c in &contentvector {
                    if c.linenumber > i.endline {
                        break;
                    } else if c.linenumber < i.startline {
                        continue;
                    }
                    i.push_str(&c.content);
                }
                if !i.is_anonymous() {
                    i.finalize();
                    if i.modified {
                        modified = true;
                    }
                }
            }

            let retfile = Specialfile {
                specialcomments: commentvector,
                sections: sectionvector,
                file: sourcefile,
                filename: sourcepath,
                targetfile,
                commentsign,
                metafile: None,
                modified,
                permissions,
            };

            return Ok(retfile);
        }
    }

    pub fn count_named_sections(&self) -> u32 {
        let mut counter = 0;
        for i in &self.sections {
            if !i.is_anonymous() {
                counter += 1;
            }
        }
        counter
    }

    pub fn update(&mut self) {
        //iterate over sections in self.sections

        let mut modified = false;
        let mut applymap: HashMap<&String, Specialfile> = HashMap::new();
        let mut applyvec = Vec::new();
        if self.metafile.is_some() {
            let metafile = &self.metafile.as_ref().unwrap();
            if metafile.modified {
                return;
            }
            if !metafile.sourcefile.is_some() {
                return;
            }
            //TODO look up what as_ref does
            match Specialfile::new(&metafile.sourcefile.as_ref().unwrap()) {
                Ok(file) => {
                    modified = self.applyfile(&file);
                }
                Err(e) => {
                    println!("failed to apply metafile sourfe, error: {}", e);
                }
            }
            return;
        }
        for i in &self.sections {
            if !i.source.is_some() {
                continue;
            }
            if let Some(source) = &i.source {
                if !applymap.contains_key(source) {
                    match Specialfile::new(source) {
                        Ok(sfile) => {
                            applymap.insert(source, sfile);
                        }
                        Err(_) => {
                            println!("error: could not open source file {}", source);
                            continue;
                        }
                    }
                }
                if let Some(sfile) = applymap.get(source) {
                    applyvec.push(sfile.clone().get_section(source).unwrap());
                }
                // if applymap.contains_key(source) {
                //     applyvec.push(
                //         applymap
                //             .get(source)
                //             .unwrap()
                //             .clone()
                //             .get_section(source)
                //             .unwrap(),
                //     );
                // }
            }
        }
        for i in applyvec.iter() {
            self.applysection(i.clone());
        }
    }

    fn get_section(&self, name: &str) -> Option<Section> {
        for i in &self.sections {
            if let Some(sname) = &i.name {
                if sname == name {
                    return Some(i.clone());
                }
            }
        }
        None
    }

    // delete section sectionname from sections
    pub fn deletesection(&mut self, sectionname: &str) -> bool {
        if let Some(index) = self.sections.iter().position(|x| match &x.name {
            Some(name) => name.eq(sectionname),
            None => false,
        }) {
            self.sections.remove(index);
            println!("deleting section {}", sectionname);
            return true;
        } else {
            return false;
        }
    }

    pub fn compile(&mut self) -> bool {
        let mut didsomething = false;
        match &mut self.metafile {
            None => {
                for i in 0..self.sections.len() {
                    didsomething = self.sections[i].compile().into() || didsomething;
                }
            }
            Some(metafile) => {
                didsomething = metafile.compile().into();
            }
        }
        didsomething
    }

    pub fn write_to_file(&mut self) {
        let targetname = &expand_tilde(&self.filename);
        let newfile = File::create(targetname);
        match newfile {
            Err(_) => {
                println!("error: could not write to file {}", &self.filename);
                panic!("write_to_file");
            }
            Ok(mut file) => match &mut self.metafile {
                None => {
                    file.write_all(self.to_string().as_bytes()).unwrap();
                }
                Some(metafile) => {
                    file.write_all(metafile.content.as_bytes()).unwrap();
                    metafile.write_to_file();
                }
            },
        }

        if let Some(permissions) = self.permissions {
            let mut perms = fs::metadata(targetname).unwrap().permissions();
            let permint = u32::from_str_radix(&format!("{}", permissions + 1000000), 8).unwrap();
            perms.set_mode(permint);
            println!("setting permissions");
            fs::set_permissions(targetname, perms).expect("failed to set permissions");
        }
    }

    // create the target file if not existing
    pub fn create_file(source: &Specialfile) -> bool {
        let targetpath = String::from(source.targetfile.clone().unwrap());
        let realtargetpath = expand_tilde(&targetpath);
        // create new file
        match &source.metafile {
            None => {
                let mut targetfile: Specialfile = Specialfile {
                    specialcomments: source.specialcomments.clone(),
                    sections: source.sections.clone(),
                    filename: realtargetpath.clone(),
                    targetfile: Option::Some(targetpath),
                    commentsign: source.commentsign.clone(),
                    file: source.file.try_clone().unwrap(),
                    metafile: None,
                    modified: source.modified,
                    permissions: source.permissions,
                };
                targetfile.write_to_file();
                return true;
            }
            Some(metafile) => {
                if metafile.modified {
                    println!(
                        "{}",
                        format!("{} modified, skipping", &source.filename).yellow()
                    );
                    return false;
                }
                OpenOptions::new()
                    .write(true)
                    .open(&realtargetpath)
                    .expect(&format!("cannot open file {}", &targetpath))
                    .write_all(metafile.content.as_bytes())
                    .expect(&format!("could not write file {}", &targetpath));
                let mut newmetafile = Metafile::from(PathBuf::from(&realtargetpath));
                newmetafile.sourcefile = Some(source.filename.clone());
                newmetafile.permissions = metafile.permissions;
                newmetafile.write_to_file();
                newmetafile.write_permissions();
                return true;
            }
        }
    }

    pub fn is_anonymous(&self) -> bool {
        let mut anonymous = true;

        for i in &self.sections {
            if !i.is_anonymous() {
                anonymous = false;
                break;
            }
        }
        anonymous
    }

    pub fn apply(&self) -> ApplyResult {
        let mut donesomething = false;
        if let Some(target) = &self.targetfile {
            if create_file(&target) {
                if Specialfile::create_file(self) {
                    println!(
                        "applied {} to create {} ",
                        &self.filename.green(),
                        &target.bold()
                    );
                    donesomething = true;
                }
            } else {
                let mut targetfile = match Specialfile::new(&expand_tilde(&target)) {
                    Ok(file) => file,
                    Err(_) => {
                        eprintln!("failed to parse {}", &target.red());
                        return ApplyResult::Error;
                    }
                };
                if targetfile.applyfile(&self) {
                    println!("applied {} to {} ", &self.filename.green(), &target.bold());
                    targetfile.write_to_file();
                    donesomething = true;
                }
            }
        } else {
            println!("{} has no target file", &self.filename.red());
            return ApplyResult::Error;
        }
        if donesomething {
            return ApplyResult::Changed;
        } else {
            return ApplyResult::Unchanged;
        }
    }

    // return true if file will be modified
    // applies other file to self
    pub fn applyfile(&mut self, inputfile: &Specialfile) -> bool {
        match &mut self.metafile {
            None => {
                if self.is_anonymous() {
                    eprintln!(
                        "{} {}",
                        "cannot apply to unmanaged file ".yellow(),
                        self.filename.yellow().bold()
                    );
                    return false;
                }
                if inputfile.metafile.is_some() {
                    eprintln!(
                        "cannot apply metafile to normal imosid file {}",
                        self.filename.bold()
                    );
                    return false;
                }

                if inputfile.is_anonymous() {
                    eprintln!(
                        "{} {}",
                        inputfile.filename.red(),
                        "is unmanaged, cannot apply"
                    );
                    return false;
                }
                //if no sections are updated, don't write anything to the file system
                let mut modified = false;

                // true if input file contains all sections that self has
                let mut allsections = true;

                for i in &self.sections {
                    allsections = false;
                    let selfname = match &i.name {
                        Some(name) => name.clone(),
                        None => {
                            continue;
                        }
                    };
                    for u in &inputfile.sections {
                        if let Some(inputname) = u.name.clone() {
                            if inputname == selfname {
                                allsections = true;
                                break;
                            }
                        } else {
                            continue;
                        }
                    }

                    if !allsections {
                        break;
                    }
                }

                if !self.modified && allsections {
                    // copy entire file contents if all sections are unmodified
                    self.sections = inputfile.sections.clone();
                    self.specialcomments = inputfile.specialcomments.clone();
                    println!(
                        "applied all sections from {} to {}",
                        inputfile.filename.bold(),
                        self.filename.bold()
                    );
                    modified = true;
                } else {
                    let mut applycounter = 0;
                    for i in &inputfile.sections {
                        if self.applysection(i.clone()) {
                            applycounter += 1;
                            modified = true;
                        }
                    }
                    if modified {
                        println!(
                            "applied {} sections from {} to {}",
                            applycounter,
                            inputfile.filename.bold(),
                            self.filename.bold()
                        );
                    } else {
                        println!(
                            "applied no sections from {} to {}{}",
                            inputfile.filename.bold().dimmed(),
                            self.filename.bold().dimmed(),
                            if self.modified {
                                " (modified)".dimmed()
                            } else {
                                "".dimmed()
                            }
                        );
                    }
                }
                return modified;
            }

            // apply entire content if file is managed by metafile
            Some(metafile) => {
                if !metafile.modified {
                    match &inputfile.metafile {
                        None => {
                            eprintln!(
                                "{}",
                                "cannot apply section file to files managed by metafiles"
                            );
                        }
                        Some(applymetafile) => {
                            if applymetafile.modified {
                                println!("source file {} modified", &applymetafile.parentfile);
                                return false;
                            }
                            if metafile.hash == applymetafile.hash {
                                println!("file {} already up to date", self.filename.bold());
                                return false;
                            }
                            metafile.content = applymetafile.content.clone();
                            metafile.hash = applymetafile.hash.clone();

                            println!(
                                "applied {} to {}",
                                inputfile.filename.bold(),
                                self.filename.bold()
                            );
                            return true;
                        }
                    }
                } else {
                    println!(
                        "{}",
                        format!("target {} modified, skipping", &self.filename.bold()).yellow()
                    );
                }
                return false;
            }
        }
    }

    fn applysection(&mut self, section: Section) -> bool {
        if let Some(_) = &self.metafile {
            eprintln!(
                "{}",
                "cannot apply individual section to file managed by metafile"
                    .red()
                    .bold()
            );
            return false;
        }
        for i in 0..self.sections.len() {
            let tmpsection = self.sections.get(i).unwrap();
            if tmpsection.is_anonymous()
                || section.is_anonymous()
                || section.modified
                || tmpsection.modified
            {
                continue;
            }
            let tmpname = &tmpsection.name.clone().unwrap();
            if tmpname == &section.name.clone().unwrap() {
                if &tmpsection.hash == &section.hash {
                    continue;
                }
                self.sections[i] = section;
                return true;
            }
        }
        return false;
    }
}

impl ToString for Specialfile {
    fn to_string(&self) -> String {
        match &self.metafile {
            None => {
                let mut retstr = String::new();
                let mut firstsection: Option<String> = Option::None;

                // respect hashbang
                // and put comments below it
                if self.targetfile.is_some() {
                    if self.sections.get(0).unwrap().is_anonymous() {
                        let firstline = String::from(
                            self.sections
                                .get(0)
                                .unwrap()
                                .content
                                .split("\n")
                                .nth(0)
                                .unwrap(),
                        );
                        let originalcontent = &self.sections.get(0).unwrap().content;

                        if Regex::new("^#!/.*").unwrap().is_match(&firstline) {
                            let mut newcontent = String::from(&firstline);
                            newcontent.push_str(&format!(
                                "\n{}... all target {}\n",
                                &self.commentsign,
                                &(self.targetfile.clone().unwrap())
                            ));
                            // reappend original section content
                            newcontent.push_str(originalcontent.trim_start_matches(&firstline));
                            firstsection = Option::Some(newcontent);
                        } else {
                            let mut newcontent = String::from(format!(
                                "{}... all target {}\n",
                                self.commentsign,
                                self.targetfile.clone().unwrap()
                            ));
                            newcontent.push_str(originalcontent);
                            firstsection = Option::Some(newcontent);
                        }
                    } else {
                        let mut newcontent = String::from(&self.commentsign);
                        newcontent.push_str("... all target");
                        newcontent.push_str(&(self.targetfile.clone().unwrap()));
                        newcontent.push('\n');

                        newcontent.push_str(&self.sections.get(0).unwrap().content);
                        firstsection = Option::Some(newcontent);
                    }
                }

                for i in &self.sections {
                    if firstsection.is_some() {
                        retstr.push_str(&firstsection.unwrap());
                        firstsection = Option::None;
                    } else {
                        retstr.push_str(&i.output(&self.commentsign));
                    }
                }
                return retstr;
            }
            Some(metafile) => {
                return metafile.content.clone();
            }
        }
    }
}

// detect comment syntax for file based on filename, extension and hashbang
fn get_comment_sign(filename: &str, firstline: &str) -> String {
    let fpath = Path::new(filename);

    let mut file_name_commentsigns: HashMap<&str, &str> = HashMap::from([
        ("dunstrc", "#"),
        ("jgmenurc", "#"),
        ("zshrc", "#"),
        ("bashrc", "#"),
        ("Xresources", "!"),
        ("xsettingsd", "#"),
        ("vimrc", "\""),
    ]);

    // get comment syntax via file name
    let fname = fpath.file_name().and_then(OsStr::to_str);
    match fname {
        Some(name) => {
            let filename = String::from(String::from(name).trim_start_matches("."));
            match file_name_commentsigns.get(filename.as_str()) {
                Some(sign) => {
                    return String::from(sign.deref());
                }
                None => {}
            }
        }
        None => {}
    }

    let mut file_type_commentsigns: HashMap<&str, &str> = HashMap::from([
        ("py", "#"),
        ("sh", "#"),
        ("zsh", "#"),
        ("bash", "#"),
        ("fish", "#"),
        ("c", "//"),
        ("cpp", "//"),
        ("rasi", "//"),
        ("desktop", "#"),
        ("conf", "#"),
        ("vim", "\""),
        ("reg", ";"),
        ("rc", "#"),
        ("ini", ";"),
        ("xresources", "!"),
    ]);

    let ext = fpath.extension().and_then(OsStr::to_str);

    // get comment syntax via file extension
    match ext {
        Some(extension) => {
            let tester = file_type_commentsigns.get(extension);
            match tester {
                Some(sign) => {
                    return String::from(sign.deref());
                }
                None => {}
            }
        }
        None => {}
    }

    // get comment syntax via #!/hashbang

    let mut file_hashbang_commentsigns: HashMap<&str, &str> = HashMap::from([
        ("python", "#"),
        ("sh", "#"),
        ("bash", "#"),
        ("zsh", "#"),
        ("fish", "#"),
        ("node", "//"),
    ]);

    match Regex::new("^#!/.*[/ ](.*)$").unwrap().captures(&firstline) {
        Some(captures) => {
            let application = captures.get(1).unwrap().as_str();
            match file_hashbang_commentsigns.get(application) {
                Some(sign) => {
                    return String::from(sign.deref());
                }
                None => {}
            }
        }
        None => {}
    }

    return String::from("#");
}

// expand tilde in path into the home folder
pub fn expand_tilde(input: &str) -> String {
    let mut retstr = String::from(input);
    if retstr.starts_with("~/") {
        retstr = String::from(format!(
            "{}/{}",
            //TODO replace deprecated home_dir
            home_dir().unwrap().into_os_string().into_string().unwrap(),
            retstr.strip_prefix("~/").unwrap()
        ));
    }
    return retstr;
}

// create file with directory creation and
// parsing of the home tilde
// MAYBETODO: support environment variables
// return false if file already exists
pub fn create_file(path: &str) -> bool {
    let realtargetname = expand_tilde(path);

    let checkpath = Path::new(&realtargetname);
    if !checkpath.is_file() {
        let bufpath = checkpath.to_path_buf();
        match bufpath.parent() {
            Some(parent) => {
                std::fs::create_dir_all(parent.to_str().unwrap()).unwrap();
            }
            None => {}
        }
        File::create(&realtargetname).unwrap();
        return true;
    } else {
        return false;
    }
}
