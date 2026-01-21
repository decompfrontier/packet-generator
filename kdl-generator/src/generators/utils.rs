use itertools::Itertools;

pub fn split_documentation(doc: &str, tab: &str, comment_str: &str, indent_level: usize) -> String {
    doc.split('\n')
        .map(|line| {
            let mut start = tab.repeat(indent_level);
            start.push_str(comment_str);
            start.push(' ');
            start.push_str(line);

            start
        })
        .join("\n")
}
