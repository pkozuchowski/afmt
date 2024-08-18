use crate::node::NodeKind;
use tree_sitter::{Node, Tree};

pub struct Visitor {
    pub formatted: String,
    pub block_indent: String,
    pub indent_level: usize,
    //pub node: &'a Node<'a>,
}

impl Visitor {
    pub fn init() -> Self {
        Visitor {
            formatted: String::new(),
            block_indent: String::from(' '),
            indent_level: 0,
        }
    }

    //https://github.com/dangmai/prettier-plugin-apex/blob/60db6549a441911a0ef25b0ecc5e61727dc92fbb/packages/prettier-plugin-apex/src/printer.ts#L612
    pub fn walk(&mut self, tree: &Tree) {
        let mut cursor = tree.walk();
        if cursor.goto_first_child() {
            loop {
                let node = &cursor.node();

                let kind = NodeKind::from_kind(node.kind());

                match kind {
                    NodeKind::ClassDeclaration => {
                        self.visit_class_node(node);
                    }
                    NodeKind::MethodDeclaration => {
                        //self.visit_method_node(node);
                    }
                    NodeKind::IfStatement => {
                        //self.visit_if_node(node);
                    }
                    NodeKind::ForLoop => {
                        //self.visit_for_node(node);
                    }
                    NodeKind::Unknown => !unimplemented!(),
                }

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    pub fn visit_class_node(&mut self, node: &Node) {
        // process sub nodes with their rewrite traits;
    }

    pub fn get_formatted(&mut self) -> String {
        self.formatted.clone()
    }

    fn push_str(&mut self, s: &str) {
        self.formatted.push_str(s);
    }
}
