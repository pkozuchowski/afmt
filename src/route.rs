use crate::{rewrite::Rewrite, struct_def::*};
use phf::phf_map;
use tree_sitter::Node;

pub static COMMON_MAP: phf::Map<
    &'static str,
    for<'a, 'tree> fn(&'a Node<'tree>) -> Box<dyn Rewrite + 'a>,
> = phf_map! {
    "class_declaration" => |node| Box::new(ClassDeclaration::new(node)),
    "method_declaration" => |node| Box::new(MethodDeclaration::new(node)),
    "block" => |node| Box::new(Block::new(node)),
    "local_variable_declaration" => |node| Box::new(LocalVariableDeclaration::new(node)),
    "array_creation_expression" => |node| Box::new(ArrayCreationExpression::new(node)),
    "array_initializer" => |node| Box::new(ArrayInitializer::new(node)),
    "expression_statement" => |node| Box::new(Statement::new(node)),
    "generic_type" => |node| Box::new(GenericType::new(node)),
    "dml_type" => |node| Box::new(DmlType::new(node)),
    "object_creation_expression" => |node| Box::new(ObjectCreationExpression::new(node)),
    "instanceof_expression" => |node| Box::new(InstanceOfExpression::new(node)),
    "annotation_argument_list" => |node| Box::new(AnnotationArgumentList::new(node)),
    "for_statement" => |node| Box::new(ForStatement::new(node)),
    "try_statement" => |node| Box::new(TryStatement::new(node)),
    "line_comment" => |node| Box::new(LineComment::new(node)),
    "method_invocation" => |node| Box::new(MethodInvocation::new(node)),
    "scoped_type_identifier" => |node| Box::new(ScopedTypeIdentifier::new(node)),
    "field_declaration" => |node| Box::new(FieldDeclaration::new(node)),
    "unary_expression" => |node| Box::new(UnaryExpression::new(node)),
    "update_expression" => |node| Box::new(UpdateExpression::new(node)),
    "dml_expression" => |node| Box::new(DmlExpression::new(node)),
    "dml_security_mode" => |node| Box::new(DmlSecurityMode::new(node)),
    "map_creation_expression" => |node| Box::new(MapCreationExpression::new(node)),
    "enum_declaration" => |node| Box::new(EnumDeclaration::new(node)),
    "enhanced_for_statement" => |node| Box::new(EnhancedForStatement::new(node)),
    "assignment_expression" => |node| Box::new(AssignmentExpression::new(node)),
    "if_statement" => |node| Box::new(IfStatement::new(node)),
    "constructor_declaration" => |node| Box::new(ConstructorDeclaration::new(node)),
    "explicit_constructor_invocation" => |node| Box::new(ExplicitConstructorInvocation::new(node)),
    "while_statement" => |node| Box::new(WhileStatement::new(node)),
    "binary_expression" => |node| Box::new(BinaryExpression::new(node)),
    "cast_expression" => |node| Box::new(CastExpression::new(node)),
    "run_as_statement" => |node| Box::new(RunAsStatement::new(node)),
    "return_statement" => |node| Box::new(ReturnStatement::new(node)),
    "dimensions_expr" => |node| Box::new(DimensionsExpr::new(node)),
    "field_access" => |node| Box::new(FieldAccess::new(node)),
    "array_access" => |node| Box::new(ArrayAccess::new(node)),
    "array_type" => |node| Box::new(ArrayType::new(node)),
    "do_statement" => |node| Box::new(DoStatement::new(node)),
    "ternary_expression" => |node| Box::new(TernaryExpression::new(node)),
    "string_literal" => |node| Box::new(Value::new(node)),
    "boolean" => |node| Box::new(Value::new(node)),
    "type_identifier" => |node| Box::new(Value::new(node)),
    "identifier" => |node| Box::new(Value::new(node)),
    "int" => |node| Box::new(Value::new(node)),
};
