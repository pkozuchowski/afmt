use crate::shape::Shape;
use crate::visitor::*;
use crate::{context::FmtContext, node_struct::*, utility::*};
use anyhow::{bail, Context, Result};
use tree_sitter::Node;

impl Visitor {
    pub fn format_class(&mut self, node: &Node, context: &FmtContext, shape: &mut Shape) {
        let n = ClassDeclaration::new(&node);
        println!("offset: {}", shape.offset);
        self.push_rewritten(n.rewrite(context, shape), &node);
        println!("offset-2: {}", shape.offset);

        self.push_block_open_line();

        let body_node = node
            .child_by_field_name("body")
            .expect("mandatory body node missing");
        self.visit_item(&body_node, context, shape);

        self.push_block_close(shape);
    }

    pub fn format_method(&mut self, node: &Node, context: &FmtContext, shape: &mut Shape) {
        let n = MethodDeclaration::new(&node);
        self.push_rewritten(n.rewrite(context, shape), &node);

        self.push_block_open_line();

        let body_node = node
            .child_by_field_name("body")
            .expect("mandatory body node missing");
        self.visit_item(&body_node, context, shape);

        self.push_block_close(shape);
    }

    pub fn format_expression_statement(
        &mut self,
        node: &Node,
        context: &FmtContext,
        shape: &mut Shape,
    ) {
        let child = node
            .named_child(0)
            .expect("ExpressionStatement mandatory child missing.");
        self.visit_item(&child, context, shape);
    }

    pub fn format_local_variable_declaration(
        &mut self,
        node: &Node,
        context: &FmtContext,
        shape: &mut Shape,
    ) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.visit_item(&child, context, shape);
        }
    }
    pub fn format_binary_expression(
        &mut self,
        node: &Node,
        context: &FmtContext,
        shape: &mut Shape,
    ) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.visit_item(&child, context, shape);
        }
    }

    pub fn format_variable_declaration(
        &mut self,
        node: &Node,
        context: &FmtContext,
        shape: &mut Shape,
    ) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.visit_item(&child, context, shape);
        }

        match node.next_named_sibling() {
            Some(sibling) if sibling.kind() == "variable_declarator" => self.push_str(", "),
            _ => {}
        }
    }

    fn push_block_open_line(&mut self) {
        self.push_str(" {\n");
    }

    fn push_block_close(&mut self, shape: &mut Shape) {
        //println!("|{:?}|", &self.block_indent);

        self.push_str(&format!("{}}}", get_indent_string(&shape.indent)));
    }

    pub fn format_if_statement(
        &mut self,
        node: &Node,
        context: &FmtContext,
        shape: &mut Shape,
    ) -> Result<()> {
        self.push_str("if");
        let condition = get_mandatory_child_by_name("condition", node)?;
        self.visit_item(&condition, context, shape);

        self.push_block_open_line();

        let consequence = get_mandatory_child_by_name("consequence", node)?;
        self.visit_item(&consequence, context, shape);

        self.push_block_close(shape);

        //let condition_node = get_mandatory_child_by_name("condition", node)?;
        //self.visit_item(&condition_node, context, shape);

        Ok(())
    }
}
