use colored::Colorize;
use tree_sitter::Node;

use crate::utility::source_code;

// `c` => child
// `cv` => child value
// `cs` => children
// `csv` => children value
// `by_n` => by name
// `by_k` => by kind
#[allow(dead_code)]
pub trait Accessor<'t> {
    fn value(&self, source_code: &str) -> String;

    fn children_vec(&self) -> Vec<Node<'t>>;

    fn try_c_by_n(&self, kind: &str) -> Option<Node<'t>>;
    fn try_c_by_k(&self, kind: &str) -> Option<Node<'t>>;
    fn try_cs_by_k(&self, kind: &str) -> Vec<Node<'t>>;

    fn first_c(&self) -> Node<'t>;
    fn try_first_c(&self) -> Option<Node<'t>>;

    fn c_by_n(&self, name: &str) -> Node<'t>;
    fn c_by_k(&self, kind: &str) -> Node<'t>;
    fn cvalue_by_n(&self, name: &str) -> String;
    fn cvalue_by_k(&self, name: &str) -> String;
    fn cs_by_k(&self, kind: &str) -> Vec<Node<'t>>;
    fn cs_by_n(&self, name: &str) -> Vec<Node<'t>>;

    fn next_named(&self) -> Node<'t>;

    // private fn;
    fn v<'a>(&self, source_code: &'a str) -> &'a str;
    fn cv_by_k<'a>(&self, name: &str, source_code: &'a str) -> &'a str;
    fn cv_by_n<'a>(&self, name: &str) -> &'a str;

    //fn try_cv_by_n<'a>(&self, name: &str, source_code: &'a str) -> Option<&'a str>;
    //fn try_cv_by_k<'a>(&self, kind: &str, source_code: &'a str) -> Option<&'a str>;
    //fn try_csv_by_k<'a>(&self, kind: &str, source_code: &'a str) -> Vec<&'a str>;
    //fn all_children_vec(&self) -> Vec<Node<'t>>;
    //fn is_comment(&self) -> bool;
}

impl<'t> Accessor<'t> for Node<'t> {
    fn next_named(&self) -> Node<'t> {
        let mut sibling = self.next_named_sibling();
        while let Some(node) = sibling {
            if !node.is_extra() {
                return node;
            }
            sibling = node.next_named_sibling();
        }
        panic!("{}: next_named node missing.", self.kind().red());
    }

    fn v<'a>(&self, source_code: &'a str) -> &'a str {
        self.utf8_text(source_code.as_bytes())
            .unwrap_or_else(|_| panic!("{}: get_value failed.", self.kind().red()))
    }

    fn value(&self, source_code: &str) -> String {
        self.v(source_code).to_string()
    }

    fn children_vec(&self) -> Vec<Node<'t>> {
        let mut cursor = self.walk();
        self.named_children(&mut cursor)
            .filter(|node| !node.is_extra())
            .collect()
    }

    fn try_c_by_k(&self, kind: &str) -> Option<Node<'t>> {
        let mut cursor = self.walk();
        let child = self.named_children(&mut cursor).find(|c| c.kind() == kind);
        child
    }

    fn try_cs_by_k(&self, kind: &str) -> Vec<Node<'t>> {
        let mut cursor = self.walk();
        self.named_children(&mut cursor)
            .filter(|c| c.kind() == kind)
            .collect()
    }

    fn try_c_by_n(&self, name: &str) -> Option<Node<'t>> {
        self.child_by_field_name(name)
    }

    fn c_by_k(&self, kind: &str) -> Node<'t> {
        self.try_c_by_k(kind).unwrap_or_else(|| {
            panic!(
                "## {}: missing mandatory kind child: {}\n ##Source_code: {}",
                self.kind().red(),
                kind.red(),
                self.value(source_code()),
            )
        })
    }

    fn try_first_c(&self) -> Option<Node<'t>> {
        let mut index = 0;
        while let Some(node) = self.named_child(index) {
            if !node.is_extra() {
                return Some(node);
            }
            index += 1;
        }
        None
    }

    fn first_c(&self) -> Node<'t> {
        let mut index = 0;
        while let Some(node) = self.named_child(index) {
            if !node.is_extra() {
                return node;
            }
            index += 1;
        }
        panic!(
            "{}: missing a mandatory child in first_c().",
            self.kind().red()
        );
    }

    fn cv_by_k<'a>(&self, name: &str, source_code: &'a str) -> &'a str {
        let child_node = self.c_by_k(name);
        child_node.v(source_code)
    }

    fn cv_by_n<'a>(&self, name: &str) -> &'a str {
        let node = self.child_by_field_name(name).unwrap_or_else(|| {
            panic!(
                "## {}: missing mandatory name child: {}\n ##Source_code: {}",
                self.kind().red(),
                name.red(),
                self.value(source_code()),
            )
        });
        node.v(source_code())
    }

    fn cvalue_by_n(&self, name: &str) -> String {
        self.cv_by_n(name).to_string()
    }

    fn cvalue_by_k(&self, name: &str) -> String {
        self.cv_by_k(name, source_code()).to_string()
    }

    fn c_by_n(&self, name: &str) -> Node<'t> {
        self.child_by_field_name(name).unwrap_or_else(|| {
            panic!(
                "## {}: missing mandatory name child: {}\n ##Source_code: {}",
                self.kind().red(),
                name.red(),
                self.value(source_code()),
            )
        })
    }

    fn cs_by_n(&self, name: &str) -> Vec<Node<'t>> {
        let mut cursor = self.walk();
        let children: Vec<Node<'t>> = self.children_by_field_name(name, &mut cursor).collect();
        if children.is_empty() {
            panic!(
                "## {}: missing mandatory name children: {}\n ##Source_code: {}",
                self.kind().red(),
                name.red(),
                self.value(source_code()),
            );
        }
        children
    }

    fn cs_by_k(&self, kind: &str) -> Vec<Node<'t>> {
        let children = self.try_cs_by_k(kind);
        if children.is_empty() {
            panic!(
                "## {}: missing mandatory kind children: {}\n ##Source_code: {}",
                self.kind().red(),
                kind.red(),
                self.value(source_code()),
            );
        }
        children
    }

    //fn try_cv_by_k<'a>(&self, kind: &str, source_code: &'a str) -> Option<&'a str> {
    //    self.try_c_by_k(kind).map(|child| child.v(source_code))
    //}

    //fn all_children_vec(&self) -> Vec<Node<'t>> {
    //    let mut cursor = self.walk();
    //    self.children(&mut cursor).collect()
    //}

    //fn try_csv_by_k<'a>(&self, kind: &str, source_code: &'a str) -> Vec<&'a str> {
    //    self.try_cs_by_k(kind)
    //        .iter()
    //        .map(|n| n.v(source_code))
    //        .collect::<Vec<&str>>()
    //}

    //fn is_comment(&self) -> bool {
    //    matches!(self.kind(), "line_comment" | "block_comment")
    //}

    //fn try_cv_by_n<'a>(&self, name: &str, source_code: &'a str) -> Option<&'a str> {
    //    self.child_by_field_name(name).map(|n| n.v(source_code))
    //}
}
