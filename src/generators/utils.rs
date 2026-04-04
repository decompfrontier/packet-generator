use itertools::Itertools;

/// Given a string inside a
/// [`doc` field](crate::intermediate::schema::Json::doc), this function
/// generates a comment string (using `comment_str`), indented to `indent_level`
/// with character `tab`.
///
/// A [`Generator`](super::Generator) may decide to use this function for
/// formatting the documentation as a comment.
///
/// This function _does not_ apply any transformation to the value passed in
/// `doc`. A [`Generator`](super::Generator) is expected to know the target
/// language's doc-string and appropriately transform it.
///
/// # Example
///
/// ```
/// # use packet_generator::generators::utils::split_documentation;
/// let doc = "# Foo";
/// let doc_string = "///";
/// let tab = " ";
/// let indent_level = 4;
///
/// let result = split_documentation(doc, tab, doc_string, indent_level);
/// assert_eq!(result, "    /// # Foo");
/// ```
#[must_use]
pub fn split_documentation(doc: &str, tab: &str, comment_str: &str, indent_level: usize) -> String {
    doc.split('\n')
        .map(|line| {
            let mut start = tab.repeat(indent_level);
            start.push_str(comment_str);
            if !line.is_empty() {
                start.push(' ');
                start.push_str(line);
            }

            start
        })
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Quick sanity check that does not use proptest.
    #[test]
    fn can_split_documentation() {
        const DOC_FIELD: &str = r#"# This is a test

Foo.
Bar. Baz. Quox.
    Bla bla bla."#;

        const TAB: &str = " ";
        const COMMENT_STR: &str = "//";
        const INDENT_LEVEL: usize = 42;

        let res = split_documentation(DOC_FIELD, TAB, COMMENT_STR, INDENT_LEVEL);

        for (line, doc_line) in res.lines().zip(DOC_FIELD.lines()) {
            let until_tab = &line[0..INDENT_LEVEL];
            assert_eq!(until_tab, TAB.repeat(INDENT_LEVEL));

            let until_comment_str = &line[(INDENT_LEVEL)..(INDENT_LEVEL + COMMENT_STR.len())];
            assert_eq!(until_comment_str, COMMENT_STR);

            let skip_whitespace = usize::from(!doc_line.is_empty());

            let rest_of_line = &line[(INDENT_LEVEL + COMMENT_STR.len() + skip_whitespace)..];

            assert_eq!(rest_of_line, doc_line);
        }
    }

    proptest! {
        #[test]
        fn can_split_arbitrary_documentation(doc in any::<String>(), tab in any::<String>(), comment_str in any::<String>(), indent in 0..1_000_000usize) {
            let res = split_documentation(&doc, &tab, &comment_str, indent);
            let tab_repeats = tab.repeat(indent);

            for (line, doc_line) in res.lines().zip(doc.lines()) {
                if !tab_repeats.is_empty() {
                    let until_tab = &line[..(tab_repeats.len())];
                    assert_eq!(until_tab, tab_repeats);
                }

                let until_comment_str = &line[(tab_repeats.len())..(tab_repeats.len() + comment_str.len())];
                assert_eq!(until_comment_str, &comment_str);

                let rest_of_line = &line[(tab_repeats.len() + comment_str.len() + 1)..];

                assert_eq!(rest_of_line, doc_line);

            }
        }
    }
}
