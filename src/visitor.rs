use anyhow::Result;
use tree_sitter::Node;

use crate::{
    config::{Context, Shape},
    node_struct::{ClassDeclaration, FieldDeclaration, MethodDeclaration, NodeKind, Rewrite},
};

#[derive(Default)]
pub struct Visitor {}

impl Visitor {
    pub fn walk(&self, node: &Node, context: &Context, parent_shape: &Shape) -> Result<String> {
        let mut results = Vec::new();

        let is_root_node = node.kind() == "parser_output";

        let shape = if is_root_node {
            Shape::new(0)
        } else {
            Shape::new(parent_shape.block_indent + 1)
        };

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            let kind = NodeKind::from_kind(child.kind());

            match kind {
                NodeKind::ClassDeclaration => {
                    let n = ClassDeclaration::new(&child, &shape);
                    results.push(n.rewrite_result(context)?);
                }
                NodeKind::FieldDeclaration => {
                    let n = FieldDeclaration::new(&child, &shape);
                    results.push(n.rewrite_result(context)?);
                }
                NodeKind::MethodDeclaration => {
                    let n = MethodDeclaration::new(&child, &shape);
                    results.push(n.rewrite_result(context)?);
                }
                //NodeKind::IfStatement => {
                //    //self.visit_if_node(node);
                //}
                //NodeKind::ForLoop => {
                //    //self.visit_for_node(node);
                //}
                NodeKind::Unknown => {
                    println!("### Unknow node: {}", child.kind());
                    println!("{}", results.join(""));
                    !unimplemented!();
                }
            }
        }

        Ok(results.join(""))
    }
}
