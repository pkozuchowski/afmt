use tree_sitter::Node;

use crate::config::Shape;

pub fn get_indent(shape: &Shape) -> String {
    let indent = "  ".repeat(shape.indent.block_indent);
    indent
}

pub fn indent_lines(prepared_code: &str, shape: &Shape) -> String {
    let indent = get_indent(shape);
    //println!("shape:{}|", shape.block_indent);
    //println!("indent:{}|", indent);

    let lines: Vec<&str> = prepared_code
        .split('\n')
        .filter(|l| !l.is_empty())
        .collect();

    let indented_lines: Vec<String> = lines
        .iter()
        .map(|line| format!("{}{}", indent, line))
        .collect();
    indented_lines.join("\n")
}

//pub fn set_global_context(source_code: String) {
//    let source_code = Box::leak(source_code.into_boxed_str());
//    let context = Context::new(source_code);
//    CONTEXT.set(context).expect("Failed to set CONTEXT");
//}

//TODO: v.s. std::sync::Once v.s. thread_local!
//pub fn get_source_code_from_context() -> &'static str {
//    CONTEXT.get().unwrap().source_code
//}

pub fn get_child_by_kind<'tree>(kind: &str, n: &Node<'tree>) -> Option<Node<'tree>> {
    let mut cursor = n.walk();
    let node = n.children(&mut cursor).find(|c| c.kind() == kind);
    node
}

pub fn get_children_by_kind<'tree>(kind: &str, n: &Node<'tree>) -> Vec<Node<'tree>> {
    let mut cursor = n.walk();
    n.children(&mut cursor)
        .filter(|c| c.kind() == kind)
        .collect()
}

pub fn get_modifiers<'tree>(n: &Node<'tree>) -> Vec<Node<'tree>> {
    if let Some(node) = get_child_by_kind("modifiers", n) {
        get_children_by_kind("modifier", &node)
    } else {
        Vec::new()
    }
}

pub fn get_parameters<'tree>(n: &Node<'tree>) -> Vec<Node<'tree>> {
    if let Some(node) = n.child_by_field_name("parameters") {
        get_children_by_kind("formal_parameter", &node)
    } else {
        Vec::new()
    }
}
