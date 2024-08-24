use crate::node_struct::{Class, NodeKind, Rewrite};
use crate::shape::Shape;
use anyhow::{anyhow, Result};
use tree_sitter::Node;

pub fn walk(node: &Node, shape: &Shape) -> Option<String> {
    let mut results = Vec::new();

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = NodeKind::from_kind(child.kind());

        match kind {
            NodeKind::ClassDeclaration => {
                let c = Class::new(&child, &shape);
                results.push(visit_class(&c)?);
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
    }

    Some(results.join(""))
}

pub fn visit_class(c: &Class) -> Option<String> {
    c.rewrite()
}
