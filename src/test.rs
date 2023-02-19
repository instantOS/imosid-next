#[cfg(test)]

mod tests {
    use crate::comment::Specialcomment;

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
        assert_eq!(comment.section.as_str(), "helloworold");
        assert_eq!(comment.argument.unwrap().as_str(), "abcdefghijk");
    }

}
