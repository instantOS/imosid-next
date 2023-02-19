#[cfg(test)]

mod tests {

    pub const FILE_CONTENT: &'static str = "#!/bin/bash

#... firstsection begin
#... firstsection hash 1F5E86D1E173F1B671B5EF32216DFF07CF973A8A7BFAFAD0AFE84BB2F29FB6C5
# comment inside the section
echo \"content of the first section\"
#... firstsection end

#... secondsection begin
#... secondsection hash E0B87AAA2E3C0A3755D20899A6FFE45B3AEA3BD43A08353B56A1037E23DEF0F8
# comment inside the section
echo \"content of the second section\"
#... secondsection end";

    use crate::comment::{CommentType, Specialcomment};
    use crate::hashable::Hashable;
    use crate::section::Section;
    use crate::files::Specialfile;

    use tempdir::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_comment() {
        let comment = Specialcomment::new("#...tester begin", "#", 20).unwrap();
        assert_eq!(comment.line, 20);
        assert_eq!(comment.section.as_str(), "tester");
    }

    #[test]
    fn test_comment_argument() {
        let comment = Specialcomment::new("#...helloworold hash abcdefghijk", "#", 21).unwrap();
        assert_eq!(comment.line, 21);
        assert_eq!(comment.ctype, CommentType::HashInfo);
        assert_eq!(comment.section.as_str(), "helloworold");
        assert_eq!(comment.argument.unwrap().as_str(), "abcdefghijk");
    }

    #[test]
    fn test_section() {
        let sectiontarget = "#... test begin
#... test hash 0DD9C99DCB5D37FB872A7FC801D8EE38922E477AE4C65F6486B02AE31981C28E
hello world
testing123
#... test end
";

        let mut testsection = Section::new(1, 10, Some(String::from("test")), None, None);
        testsection.push_str("hello world");
        testsection.push_str("testing123");
        testsection.finalize();
        testsection.compile();
        assert_eq!(
            testsection.hash.as_str(),
            "0DD9C99DCB5D37FB872A7FC801D8EE38922E477AE4C65F6486B02AE31981C28E"
        );
        assert_eq!(testsection.output(&"#").as_str(), sectiontarget);
    }

    #[test]
    fn testfile() {
        let tmp_dir = TempDir::new("imosidtest").unwrap();
        let testpath = tmp_dir.path().join("testfile.sh");
        let mut testfile = File::create(&testpath).unwrap();
        testfile.write_all(FILE_CONTENT.as_bytes()).unwrap();

        let mut testfile = Specialfile::from_pathbuf(&testpath).unwrap();
        let mut sectioncount = 0;

        for section in testfile.sections {
            if !section.is_anonymous() {
                sectioncount += 1;
                // not sure if this is too complicated...
                assert!(vec!["firstsection", "secondsection"].contains(&section.name.unwrap().as_str()));
            }
        }

        assert_eq!(sectioncount, 2);

    }
}
