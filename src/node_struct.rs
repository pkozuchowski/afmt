use crate::context::FmtContext;
use crate::shape::Shape;
use crate::utility::*;
use crate::{define_struct, define_struct_and_enum};
use anyhow::{Context, Result};
use tree_sitter::Node;

pub trait Rewrite {
    fn rewrite(&self, context: &FmtContext, shape: &Shape) -> Option<String> {
        self.rewrite_result(context, shape).ok()
    }

    fn rewrite_result(&self, context: &FmtContext, shape: &Shape) -> Result<String>;
}

define_struct_and_enum!(
    true; ClassDeclaration => "class_declaration",
    true; FieldDeclaration => "field_declaration",
    true; MethodDeclaration => "method_declaration",
    false; EmptyNode => "block" | "class_body",
    false; ExpressionStatement => "expression_statement",
    true; Value => "boolean" | "int" | "identifier"  |  "string_literal" | "," | ";",
    true; ValueSpace => "type_identifier",
    true; SpaceValueSpace => "assignment_operator",
    false; BinaryExpression => "binary_expression",
    false; LocalVariableDeclaration => "local_variable_declaration",
    true; VariableDeclarator => "variable_declarator",
    false; IfStatement => "if_statement",
    false; ParenthesizedExpression => "parenthesized_expression"
);

impl<'a, 'tree> ClassDeclaration<'a, 'tree> {}

impl<'a, 'tree> Rewrite for ClassDeclaration<'a, 'tree> {
    fn rewrite_result(&self, context: &FmtContext, shape: &Shape) -> Result<String> {
        let mut result = String::new();

        let modifiers_value = get_modifiers_value(self.as_ast_node(), context.source_code);
        result.push_str(&modifiers_value);
        result.push_str(" class ");

        let name_node = self
            .as_ast_node()
            .child_by_field_name("name")
            .context("mandatory name field missing")?;
        let name_node_value = get_value(&name_node, context.source_code);

        result.push_str(name_node_value);
        Ok(result)
    }
}

impl<'a, 'tree> Rewrite for MethodDeclaration<'a, 'tree> {
    fn rewrite_result(&self, context: &FmtContext, shape: &Shape) -> Result<String> {
        let mut result = String::new();
        //result.push_str(&get_indent_string(&shape.indent));

        let modifier_nodes = get_modifiers(self.as_ast_node());
        let modifiers_doc = modifier_nodes
            .iter()
            .map(|n| get_value(n, context.source_code))
            .collect::<Vec<&str>>()
            .join(" ");

        result.push_str(&modifiers_doc);
        result.push(' ');

        let type_node = self
            .as_ast_node()
            .child_by_field_name("type")
            .context("mandatory type field missing")?;
        let type_node_value = get_value(&type_node, context.source_code);
        result.push_str(type_node_value);
        result.push(' ');

        let name_node = self
            .as_ast_node()
            .child_by_field_name("name")
            .context("mandatory name field missing")?;
        let name_node_value = get_value(&name_node, context.source_code);
        result.push_str(name_node_value);

        result.push('(');
        let parameters_node = get_parameters(self.as_ast_node());
        let parameters_doc = parameters_node
            .iter()
            .map(|n| {
                let type_node = n.child_by_field_name("type").unwrap();
                let name_node = n.child_by_field_name("name").unwrap();
                let type_str = type_node
                    .utf8_text(context.source_code.as_bytes())
                    .ok()
                    .unwrap();
                let name_str = name_node
                    .utf8_text(context.source_code.as_bytes())
                    .ok()
                    .unwrap();
                let r = format!("{} {}", type_str, name_str);
                r
            })
            .collect::<Vec<String>>()
            .join(", ");

        result.push_str(&parameters_doc);
        result.push(')');

        Ok(result)
    }
}

impl<'a, 'tree> Rewrite for FieldDeclaration<'a, 'tree> {
    fn rewrite_result(&self, context: &FmtContext, shape: &Shape) -> Result<String> {
        let mut result = String::new();
        //result.push_str(&get_indent_string(&shape.indent));

        let modifier_nodes = get_modifiers(self.as_ast_node());
        let modifiers_doc = modifier_nodes
            .iter()
            .map(|n| {
                n.utf8_text(context.source_code.as_bytes())
                    .ok()
                    .unwrap_or_default()
            })
            .collect::<Vec<&str>>()
            .join(" ");

        result.push_str(&modifiers_doc);

        result.push(' ');

        let type_node = self
            .as_ast_node()
            .child_by_field_name("type")
            .context("mandatory type field missing")?;
        let type_node_value = get_value(&type_node, context.source_code);
        result.push_str(type_node_value);

        result.push(' ');

        let name_node = self
            .as_ast_node()
            .child_by_field_name("declarator")
            .context("mandatory declarator field missing")?
            .child_by_field_name("name")
            .context("mandatory name field missing")?;
        let name_node_value = get_value(&name_node, context.source_code);
        result.push_str(name_node_value);
        //let mut result = indent_lines(&result, shape);
        //println!("fieldD: result |{}|", result);
        Ok(result)
    }
}

impl<'a, 'tree> Rewrite for Value<'a, 'tree> {
    fn rewrite_result(&self, context: &FmtContext, shape: &Shape) -> Result<String> {
        let mut result = String::new();
        //result.push_str(&get_indent_string(&shape.indent));

        let name_node_value = get_value(self.as_ast_node(), context.source_code);
        result.push_str(name_node_value);
        Ok(result)
    }
}

impl<'a, 'tree> Rewrite for SpaceValueSpace<'a, 'tree> {
    fn rewrite_result(&self, context: &FmtContext, shape: &Shape) -> Result<String> {
        let mut result = String::from(' ');
        let name_node_value = get_value(self.as_ast_node(), context.source_code);
        result.push_str(name_node_value);
        result.push(' ');
        Ok(result)
    }
}

impl<'a, 'tree> Rewrite for ValueSpace<'a, 'tree> {
    fn rewrite_result(&self, context: &FmtContext, shape: &Shape) -> Result<String> {
        let mut result = String::new();

        let name_node_value = get_value(self.as_ast_node(), context.source_code);
        result.push_str(name_node_value);
        result.push(' ');
        Ok(result)
    }
}

//impl<'a, 'tree> Rewrite for IfStatement<'a, 'tree> {
//    fn rewrite_result(&self, context: &FmtContext, shape: &Shape) -> Result<String> {
//        let mut result = String::new();
//        result.push_str("if");
//        let condition_node = get_mandatory_child_by_name("condition", self.as_ast_node());
//
//        Ok(result)
//    }
//}
