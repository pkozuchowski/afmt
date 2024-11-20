use colored::Colorize;
use serde::Serialize;
use tree_sitter::Node;

use crate::{
    accessor::Accessor, data_model::*, doc::DocRef, doc_builder::DocBuilder, utility::source_code,
};

#[derive(Debug, Serialize)]
pub enum RootMember {
    Class(Box<ClassDeclaration>),
    Enum(Box<EnumDeclaration>),
    Interface(Box<InterfaceDeclaration>),
    Trigger(Box<TriggerDeclaration>),
}

impl<'a> DocBuild<'a> for RootMember {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            RootMember::Class(n) => {
                result.push(n.build(b));
            }
            RootMember::Enum(n) => {
                result.push(n.build(b));
            }
            RootMember::Interface(n) => {
                result.push(n.build(b));
            }
            RootMember::Trigger(n) => {
                result.push(n.build(b));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub enum ClassMember {
    Field(Box<FieldDeclaration>),
    NestedClass(Box<ClassDeclaration>),
    Method(Box<MethodDeclaration>),
    Block(Box<Block>),
    Interface(Box<InterfaceDeclaration>),
    Enum(EnumDeclaration),
    Static(StaticInitializer),
    Constructor(ConstructorDeclaration),
    //SemiColumn,
}

impl ClassMember {
    pub fn new(n: Node) -> Self {
        match n.kind() {
            "field_declaration" => Self::Field(Box::new(FieldDeclaration::new(n))),
            "class_declaration" => Self::NestedClass(Box::new(ClassDeclaration::new(n))),
            "method_declaration" => Self::Method(Box::new(MethodDeclaration::new(n))),
            "interface_declaration" => Self::Interface(Box::new(InterfaceDeclaration::new(n))),
            "block" => Self::Block(Box::new(Block::new(n))),
            "constructor_declaration" => Self::Constructor(ConstructorDeclaration::new(n)),
            "enum_declaration" => Self::Enum(EnumDeclaration::new(n)),
            "static_initializer" => Self::Static(StaticInitializer::new(n)),
            _ => panic!("## unknown node: {} in UnannotatedType ", n.kind().red()),
        }
    }
}

impl<'a> DocBuild<'a> for ClassMember {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Field(field_decl) => {
                result.push(field_decl.build(b));
            }
            Self::NestedClass(class_decl) => {
                result.push(class_decl.build(b));
            }
            Self::Method(method) => {
                result.push(method.build(b));
            }
            Self::Interface(i) => {
                result.push(i.build(b));
            }
            Self::Block(block) => {
                result.push(block.build(b));
            }
            Self::Constructor(c) => {
                result.push(c.build(b));
            }
            Self::Enum(en) => {
                result.push(en.build(b));
            }
            Self::Static(s) => {
                result.push(s.build(b));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub enum UnannotatedType {
    Simple(SimpleType),
    Array(Box<ArrayType>),
}

impl UnannotatedType {
    pub fn new(n: Node) -> Self {
        match n.kind() {
            "type_identifier"
            | "void_type"
            | "generic_type"
            | "java_type"
            | "scoped_type_identifier" => Self::Simple(SimpleType::new(n)),
            "array_type" => Self::Array(Box::new(ArrayType::new(n))),
            _ => panic!("## unknown node: {} in UnannotatedType ", n.kind().red()),
        }
    }
}

impl<'a> DocBuild<'a> for UnannotatedType {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Simple(n) => result.push(n.build(b)),
            Self::Array(n) => result.push(n.build(b)),
        }
    }
}

#[derive(Debug, Serialize)]
pub enum SimpleType {
    Identifier(String),
    Void(VoidType),
    Generic(GenericType),
    Scoped(ScopedTypeIdentifier),
    Java(JavaType),
}

impl SimpleType {
    pub fn new(n: Node) -> Self {
        match n.kind() {
            "type_identifier" => Self::Identifier(n.value(source_code())),
            "void_type" => Self::Void(VoidType::new(n)),
            "java_type" => Self::Java(JavaType::new(n)),
            "generic_type" => Self::Generic(GenericType::new(n)),
            "scoped_type_identifier" => Self::Scoped(ScopedTypeIdentifier::new(n)),
            _ => panic!("## unknown node: {} in SimpleType", n.kind().red()),
        }
    }
}

impl<'a> DocBuild<'a> for SimpleType {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Identifier(n) => {
                result.push(b.txt(n));
            }
            Self::Java(n) => {
                result.push(n.build(b));
            }
            Self::Void(n) => {
                result.push(b.txt(&n.value));
            }
            Self::Generic(n) => {
                result.push(n.build(b));
            }
            Self::Scoped(n) => {
                result.push(n.build(b));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub enum VariableInitializer {
    Exp(Expression),
    Array(Box<ArrayInitializer>),
}

impl VariableInitializer {
    pub fn new(n: Node) -> Self {
        match n.kind() {
            "array_initializer" => Self::Array(Box::new(ArrayInitializer::new(n))),
            _ => Self::Exp(Expression::new(n)),
        }
    }
}

impl<'a> DocBuild<'a> for VariableInitializer {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Exp(exp) => {
                result.push(exp.build(b));
            }
            Self::Array(a) => {
                result.push(a.build(b));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub enum Expression {
    Assignment(Box<AssignmentExpression>),
    Binary(Box<BinaryExpression>),
    Primary(Box<PrimaryExpression>),
    Update(UpdateExpression),
    Unary(UnaryExpression),
    Dml(Box<DmlExpression>),
    Te(Box<TernaryExpression>),
    Cast(Box<CastExpression>),
    Instance(Box<InstanceOfExpression>),
}

impl Expression {
    pub fn new(n: Node) -> Self {
        match n.kind() {
            "assignment_expression" => Self::Assignment(Box::new(AssignmentExpression::new(n))),
            "binary_expression" => Self::Binary(Box::new(BinaryExpression::new(n))),
            "int"
            | "decimal_floating_point_literal"
            | "boolean"
            | "identifier"
            | "null_literal"
            | "method_invocation"
            | "parenthesized_expression"
            | "object_creation_expression"
            | "array_access"
            | "field_access"
            | "string_literal"
            | "version_expression"
            | "java_field_access"
            | "this"
            | "array_creation_expression" => Self::Primary(Box::new(PrimaryExpression::new(n))),
            "update_expression" => Self::Update(UpdateExpression::new(n)),
            "unary_expression" => Self::Unary(UnaryExpression::new(n)),
            "dml_expression" => Self::Dml(Box::new(DmlExpression::new(n))),
            "ternary_expression" => Self::Te(Box::new(TernaryExpression::new(n))),
            "cast_expression" => Self::Cast(Box::new(CastExpression::new(n))),
            "instanceof_expression" => Self::Instance(Box::new(InstanceOfExpression::new(n))),
            _ => panic!("## unknown node: {} in Expression", n.kind().red()),
        }
    }
}

impl<'a> DocBuild<'a> for Expression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Assignment(n) => {
                result.push(n.build(b));
            }
            Self::Binary(n) => {
                result.push(n.build(b));
            }
            Self::Primary(n) => {
                result.push(n.build(b));
            }
            Self::Update(n) => {
                result.push(n.build(b));
            }
            Self::Unary(n) => {
                result.push(n.build(b));
            }
            Self::Dml(n) => {
                result.push(n.build(b));
            }
            Self::Te(n) => {
                result.push(n.build(b));
            }
            Self::Cast(n) => {
                result.push(n.build(b));
            }
            Self::Instance(n) => {
                result.push(n.build(b));
            }
        }
    }
}

//primary_expression: ($) =>
//  choice(
//    $._literal,
//    $.class_literal,
//    $.this,
//    $.identifier,
//    $.parenthesized_expression,
//    $.object_creation_expression,
//    $.field_access,
//    $.java_field_access,
//    $.array_access,
//    $.method_invocation,
//    $.array_creation_expression,
//    $.map_creation_expression,
//    $.query_expression,
//    $.version_expression
//  ),

#[derive(Debug, Serialize)]
pub enum PrimaryExpression {
    Literal(Literal_),
    Identifier(String),
    Method(MethodInvocation),
    Parenth(ParenthesizedExpression),
    Obj(ObjectCreationExpression),
    Field(FieldAccess),
    Array(Box<ArrayAccess>),
    ArrayCreation(ArrayCreationExpression),
    Version(VersionExpression),
    This(This),
    Java(JavaFieldAccess),
}

impl PrimaryExpression {
    pub fn new(n: Node) -> Self {
        match n.kind() {
            "int"
            | "decimal_floating_point_literal"
            | "boolean"
            | "null_literal"
            | "string_literal" => Self::Literal(Literal_::new(n)),
            "identifier" => Self::Identifier(n.value(source_code())),
            "method_invocation" => Self::Method(MethodInvocation::new(n)),
            "parenthesized_expression" => Self::Parenth(ParenthesizedExpression::new(n)),
            "object_creation_expression" => Self::Obj(ObjectCreationExpression::new(n)),
            "field_access" => Self::Field(FieldAccess::new(n)),
            "array_access" => Self::Array(Box::new(ArrayAccess::new(n))),
            "array_creation_expression" => Self::ArrayCreation(ArrayCreationExpression::new(n)),
            "version_expression" => Self::Version(VersionExpression::new(n)),
            "java_field_access" => Self::Java(JavaFieldAccess::new(n)),
            "this" => Self::This(This::new(n)),
            _ => panic!("## unknown node: {} in PrimaryExpression", n.kind().red()),
        }
    }
}

impl<'a> DocBuild<'a> for PrimaryExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Literal(n) => {
                result.push(n.build(b));
            }
            Self::Identifier(n) => {
                result.push(b.txt(n));
            }
            Self::Method(n) => {
                result.push(n.build(b));
            }
            Self::Parenth(n) => {
                result.push(n.build(b));
            }
            Self::Obj(n) => {
                result.push(n.build(b));
            }
            Self::Field(n) => {
                result.push(n.build(b));
            }
            Self::Array(n) => {
                result.push(n.build(b));
            }
            Self::ArrayCreation(n) => {
                result.push(n.build(b));
            }
            Self::Version(n) => {
                result.push(n.build(b));
            }
            Self::Java(n) => {
                result.push(n.build(b));
            }
            Self::This(n) => {
                result.push(n.build(b));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub enum Literal_ {
    Bool(String),
    Null,
    Int(String),
    Decimal(String),
    Str(String),
}

impl Literal_ {
    pub fn new(n: Node) -> Self {
        match n.kind() {
            "boolean" => Self::Bool(n.value(source_code()).to_lowercase()),
            "null_literal" => Self::Null,
            "int" => Self::Int(n.value(source_code())),
            "string_literal" => Self::Str(n.value(source_code())),
            "decimal_floating_point_literal" => Self::Decimal(n.value(source_code())),
            _ => panic!("## unknown node: {} in Literal", n.kind().red()),
        }
    }
}

impl<'a> DocBuild<'a> for Literal_ {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Bool(n) => {
                result.push(b.txt(n));
            }
            Self::Null => {
                result.push(b.txt("null"));
            }
            Self::Int(n) => {
                result.push(b.txt(n));
            }
            Self::Decimal(n) => {
                result.push(b.txt(n));
            }
            Self::Str(n) => {
                result.push(b.txt(n));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub enum Modifier {
    Abstract,
    Final,
    Global,
    InheritedSharing,
    Override,
    Private,
    Protected,
    Public,
    Static,
    TestMethod,
    Transient,
    Virtual,
    Webservice,
    WithSharing,
    WithoutSharing,
}

impl Modifier {
    pub fn new(n: Node) -> Self {
        let kind = n.kind();
        match kind {
            "global" => Self::Global,
            "public" => Self::Public,
            "with_sharing" => Self::WithSharing,
            "without_sharing" => Self::WithoutSharing,
            "private" => Self::Private,
            "override" => Self::Override,
            "static" => Self::Static,
            "final" => Self::Final,
            "virtual" => Self::Virtual,
            "abstract" => Self::Abstract,
            "inherited_sharing" => Self::InheritedSharing,
            "protected" => Self::Protected,
            "test method" => Self::TestMethod,
            "transient" => Self::Transient,
            "webservice" => Self::Webservice,
            _ => panic!("## unknown node: {} in Modifier", kind),
        }
    }
}

impl<'a> DocBuild<'a> for Modifier {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Abstract => {
                result.push(b.txt("abstract"));
            }
            Self::Final => {
                result.push(b.txt("final"));
            }
            Self::Global => {
                result.push(b.txt("global"));
            }
            Self::InheritedSharing => {
                result.push(b.txt("inherited sharing"));
            }
            Self::Override => {
                result.push(b.txt("override"));
            }
            Self::Private => {
                result.push(b.txt("private"));
            }
            Self::Protected => {
                result.push(b.txt("protected"));
            }
            Self::Public => {
                result.push(b.txt("public"));
            }
            Self::Static => {
                result.push(b.txt("static"));
            }
            Self::TestMethod => {
                result.push(b.txt("testmethod"));
            }
            Self::Transient => {
                result.push(b.txt("transient"));
            }
            Self::Virtual => {
                result.push(b.txt("virtual"));
            }
            Self::Webservice => {
                result.push(b.txt("webserivce"));
            }
            Self::WithSharing => {
                result.push(b.txt("with sharing"));
            }
            Self::WithoutSharing => {
                result.push(b.txt("without sharing"));
            }
        }
    }
}

//statement: ($) =>
//  choice(
//    $.declaration,
//    $.expression_statement,
//    $.if_statement,
//    $.while_statement,
//    $.for_statement,
//    $.enhanced_for_statement,
//    $.block,
//    ";",
//    $.do_statement,
//    $.break_statement,
//    $.continue_statement,
//    $.return_statement,
//    $.switch_expression,
//    $.local_variable_declaration,
//    $.throw_statement,
//    $.try_statement,
//    $.run_as_statement

#[derive(Debug, Serialize)]
pub enum Statement {
    If(Box<IfStatement>),
    Exp(Expression),
    Local(LocalVariableDeclaration),
    Block(Box<Block>),
    For(Box<ForStatement>),
    EnhancedFor(Box<EnhancedForStatement>),
    Run(RunAsStatement),
    Do(Box<DoStatement>),
    While(Box<WhileStatement>),
    Return(ReturnStatement),
    Try(Box<TryStatement>),
    Throw(ThrowStatement),
    Break(BreakStatement),
    Continue(ContinueStatement),
    Switch(Box<SwitchExpression>),
    SemiColumn,
}

impl Statement {
    pub fn new(n: Node) -> Self {
        match n.kind() {
            "if_statement" => Self::If(Box::new(IfStatement::new(n))),
            "expression_statement" => Self::Exp(Expression::new(n.first_c())),
            "local_variable_declaration" => Self::Local(LocalVariableDeclaration::new(n)),
            "block" => Self::Block(Box::new(Block::new(n))),
            "for_statement" => Self::For(Box::new(ForStatement::new(n))),
            "enhanced_for_statement" => Self::EnhancedFor(Box::new(EnhancedForStatement::new(n))),
            "run_as_statement" => Self::Run(RunAsStatement::new(n)),
            "do_statement" => Self::Do(Box::new(DoStatement::new(n))),
            "while_statement" => Self::While(Box::new(WhileStatement::new(n))),
            "return_statement" => Self::Return(ReturnStatement::new(n)),
            "try_statement" => Self::Try(Box::new(TryStatement::new(n))),
            "throw_statement" => Self::Throw(ThrowStatement::new(n)),
            "break_statement" => Self::Break(BreakStatement::new(n)),
            "continue_statement" => Self::Continue(ContinueStatement::new(n)),
            "switch_expression" => Self::Switch(Box::new(SwitchExpression::new(n))),
            ";" => Self::SemiColumn,
            _ => panic!("## unknown node: {} in Statement", n.kind().red()),
        }
    }
    pub fn is_block(&self) -> bool {
        matches!(self, Statement::Block(_))
    }
}

impl<'a> DocBuild<'a> for Statement {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::If(n) => {
                result.push(n.build(b));
            }
            Self::Exp(n) => {
                result.push(n.build(b));
                result.push(b.txt(";"));
            }
            Self::Local(n) => {
                result.push(n.build(b));
                result.push(b.txt(";"));
            }
            Self::Block(n) => {
                result.push(n.build(b));
            }
            Self::For(n) => {
                result.push(n.build(b));
            }
            Self::EnhancedFor(n) => {
                result.push(n.build(b));
            }
            Self::Run(n) => {
                result.push(n.build(b));
            }
            Self::Do(n) => {
                result.push(n.build(b));
            }
            Self::While(n) => {
                result.push(n.build(b));
            }
            Self::Return(n) => {
                result.push(n.build(b));
            }
            Self::Try(n) => {
                result.push(n.build(b));
            }
            Self::Throw(n) => {
                result.push(n.build(b));
            }
            Self::Break(n) => {
                result.push(n.build(b));
            }
            Self::Continue(n) => {
                result.push(n.build(b));
            }
            Self::Switch(n) => {
                result.push(n.build(b));
            }
            Self::SemiColumn => {
                result.push(b.txt(";"));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub enum Type {
    Unannotated(UnannotatedType),
}

impl Type {
    pub fn new(n: Node) -> Self {
        match n.kind() {
            "type_identifier"
            | "void_type"
            | "generic_type"
            | "scoped_type_identifier"
            | "java_type" => Self::Unannotated(UnannotatedType::Simple(SimpleType::new(n))),
            _ => panic!("## unknown node: {} in Type", n.kind().red()),
        }
    }
}

impl<'a> DocBuild<'a> for Type {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Unannotated(u) => {
                result.push(u.build(b));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub enum PropertyNavigation {
    SafeNavigationOperator,
    Dot,
}

impl<'a> DocBuild<'a> for PropertyNavigation {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::SafeNavigationOperator => {
                result.push(b.txt("?."));
            }
            Self::Dot => {
                result.push(b.txt("."));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub enum AnnotationArgumentList {
    Nil,
    Value(String),
    KeyValues(Vec<AnnotationKeyValue>),
}

impl AnnotationArgumentList {
    pub fn new(n: Node) -> Self {
        if n.named_child_count() == 0 {
            return Self::Nil;
        }

        let key_values = n.try_cs_by_k("annotation_key_value");

        if key_values.is_empty() {
            Self::Value(n.cvalue_by_n("value", source_code()))
        } else {
            let key_values = key_values
                .into_iter()
                .map(AnnotationKeyValue::new)
                .collect();
            Self::KeyValues(key_values)
        }
    }
}

impl<'a> DocBuild<'a> for AnnotationArgumentList {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Nil => {}
            Self::Value(v) => {
                result.push(b.txt("("));
                result.push(b.txt(v));
                result.push(b.txt(")"));
            }
            Self::KeyValues(vec) => {
                if !vec.is_empty() {
                    let docs = b.to_docs(vec);

                    let doc = b.concat(vec![
                        b.txt("("),
                        b.intersperse_single_line(&docs, " "),
                        b.txt(")"),
                    ]);

                    result.push(doc);
                }
            }
        }
    }
}

// Generic struct to associate a body member with trailing newline information
#[derive(Debug, Serialize)]
pub struct BodyMember<M> {
    pub member: M,
    pub has_trailing_newline: bool,
}

#[derive(Debug, Serialize)]
pub enum TriggerEvent {
    BeforeInsert,
    BeforeUpdate,
    BeforeDelete,
    AfterInsert,
    AfterUpdate,
    AfterDelete,
    AfterUndelete,
}

impl TriggerEvent {
    pub fn new(n: Node) -> Self {
        match n.kind() {
            "before_insert" => Self::BeforeInsert,
            "before_Update" => Self::BeforeUpdate,
            "before_delete" => Self::BeforeDelete,
            "after_insert" => Self::AfterInsert,
            "after_update" => Self::AfterUpdate,
            "after_delete" => Self::AfterDelete,
            "after_undelete" => Self::AfterUndelete,
            _ => panic!("## unknown node: {} in TriggerEvent", n.kind().red()),
        }
    }
}

impl<'a> DocBuild<'a> for TriggerEvent {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::BeforeInsert => {
                result.push(b.txt("before insert"));
            }
            Self::BeforeUpdate => {
                result.push(b.txt("before update"));
            }
            Self::BeforeDelete => {
                result.push(b.txt("before delete"));
            }
            Self::AfterInsert => {
                result.push(b.txt("after insert"));
            }
            Self::AfterUpdate => {
                result.push(b.txt("after update"));
            }
            Self::AfterDelete => {
                result.push(b.txt("after delete"));
            }
            Self::AfterUndelete => {
                result.push(b.txt("after undelete"));
            }
        }
    }
}
