use crate::context::Context;
use crate::extension::NodeUtilities;
use crate::shape::Shape;
use crate::utility::get_indent;
use tree_sitter::Node;

#[derive(Debug)]
pub enum NodeKind {
    ClassDeclaration,
    MethodDeclaration,
    IfStatement,
    ForLoop,
    Unknown,
}

impl NodeKind {
    pub fn from_kind(kind: &str) -> NodeKind {
        match kind {
            "class_declaration" => NodeKind::ClassDeclaration,
            "method_declaration" => NodeKind::MethodDeclaration,
            "if_statement" => NodeKind::IfStatement,
            "for_statement" => NodeKind::ForLoop,
            _ => NodeKind::Unknown,
        }
    }
}

pub trait Rewrite {
    fn rewrite(&self, shape: &Shape, context: &Context) -> Option<String>;

    //fn rewrite_result(&self) -> RewriteResult {
    //    self.rewrite(context, shape).unknown_error()
    //}
}

pub struct Class<'a, 'tree> {
    inner: &'a Node<'tree>,
}

impl<'a, 'tree> Class<'a, 'tree> {
    pub fn new(node: &'a Node<'tree>) -> Self {
        Class { inner: node }
    }

    pub fn as_ast_node(&self) -> &'a Node<'tree> {
        self.inner
    }

    pub fn get_modifiers(&self) -> Vec<Node<'tree>> {
        if let Some(n) = self.as_ast_node().get_child_by_kind("modifiers") {
            n.get_children_by_kind("modifier")
        } else {
            Vec::new()
        }
    }

    pub fn format_body(&self, shape: &Shape) -> Option<String> {
        Some(String::new())
    }
}

impl<'a, 'tree> Rewrite for Class<'a, 'tree> {
    fn rewrite(&self, shape: &Shape, context: &Context) -> Option<String> {
        let modifier_nodes = self.get_modifiers();
        let modifiers_doc = modifier_nodes
            .iter()
            .map(|n| {
                n.utf8_text(context.source_code.as_bytes())
                    .ok()
                    .unwrap_or_default()
            })
            .collect::<Vec<&str>>()
            .join(" ");

        let mut result = String::new();
        result.push_str(&modifiers_doc);
        result.push(' ');

        let name_node = self.as_ast_node().child_by_field_name("name")?;
        let name_node_value = name_node.utf8_text(context.source_code.as_bytes()).ok()?;

        let indent = get_indent(shape);
        result.push_str(name_node_value);
        result.push_str(" {\n");

        let mut body_shape = shape.clone();
        body_shape.block_indent += 1;
        self.format_body(&body_shape);

        result.push('}');

        println!("class result: {}", result);
        Some(result)
    }
}
