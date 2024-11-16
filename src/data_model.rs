use crate::{
    accessor::Accessor,
    doc::DocRef,
    doc_builder::DocBuilder,
    enum_def::*,
    utility::{assert_check, has_trailing_new_line, source_code},
};
use colored::Colorize;
use serde::Serialize;
use std::fmt::Debug;
use tree_sitter::{Node, Point, Range};

pub trait DocBuild<'a> {
    fn build(&self, b: &'a DocBuilder<'a>) -> DocRef<'a> {
        let mut result: Vec<DocRef<'a>> = Vec::new();
        self.build_inner(b, &mut result);
        b.concat(result)
    }

    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>);
}

#[derive(Debug, Default, Serialize)]
pub struct Root {
    pub members: Vec<RootMember>,
}

impl Root {
    pub fn new(node: Node) -> Self {
        assert_check(node, "parser_output");
        let mut root = Root::default();

        for c in node.children_vec() {
            match c.kind() {
                "class_declaration" => root
                    .members
                    .push(RootMember::Class(Box::new(ClassDeclaration::new(c)))),
                _ => panic!("## unknown node: {} in Root ", c.kind().red()),
            }
        }
        root
    }
}

impl<'a> DocBuild<'a> for Root {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let member_docs = b.to_docs(&self.members);
        let body_doc = b.intersperse_with_sep_and_softline(&member_docs, "");
        result.push(body_doc);
        result.push(b.nl());
    }
}

#[derive(Debug, Serialize)]
pub struct ClassDeclaration {
    pub buckets: Option<CommentBuckets>,
    pub modifiers: Option<Modifiers>,
    pub name: String,
    pub superclass: Option<SuperClass>,
    pub interface: Option<Interface>,
    pub body: ClassBody,
    pub range: DataRange,
}

impl ClassDeclaration {
    pub fn new(node: Node) -> Self {
        assert_check(node, "class_declaration");
        let buckets = None;

        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let name = node.cvalue_by_n("name", source_code());
        let superclass = node.try_c_by_k("superclass").map(|n| SuperClass::new(n));
        let interface = node.try_c_by_k("interfaces").map(|n| Interface::new(n));
        let body = ClassBody::new(node.c_by_n("body"));
        let range = DataRange::from(node.range());

        Self {
            buckets,
            modifiers,
            name,
            superclass,
            interface,
            body,
            range,
        }
    }
}

impl<'a> DocBuild<'a> for ClassDeclaration {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.modifiers {
            result.push(n.build(b));
        }

        result.push(b.txt_("class"));
        result.push(b.txt(&self.name));

        if let Some(ref n) = self.superclass {
            result.push(n.build(b));
        }

        if let Some(ref n) = self.interface {
            result.push(n.build(b));
        }

        result.push(b.txt(" {"));

        if self.body.class_members.is_empty() {
            result.push(b.nl());
        } else {
            result.push(b.add_indent_level(b.nl()));
            let body_doc = self.body.build(b);
            let indented_body = b.add_indent_level(body_doc);
            result.push(indented_body);
            result.push(b.nl());
        }

        result.push(b.txt("}"));
    }
}

#[derive(Debug, Serialize)]
pub struct MethodDeclaration {
    pub modifiers: Option<Modifiers>,
    pub type_: UnnanotatedType,
    pub name: String,
    pub formal_parameters: FormalParameters,
    pub body: Option<Block>,
    //pub dimentions
}

impl MethodDeclaration {
    pub fn new(node: Node) -> Self {
        assert_check(node, "method_declaration");

        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let type_ = UnnanotatedType::new(node.c_by_n("type"));
        let name = node.cvalue_by_n("name", source_code());
        let formal_parameters = FormalParameters::new(node.c_by_n("parameters"));
        let body = node.try_c_by_n("body").map(|n| Block::new(n));

        Self {
            modifiers,
            type_,
            name,
            formal_parameters,
            body,
        }
    }
}

impl<'a> DocBuild<'a> for MethodDeclaration {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.modifiers {
            result.push(n.build(b));
        }

        result.push(&self.type_.build(b));
        result.push(b._txt(&self.name));
        result.push(self.formal_parameters.build(b));
        result.push(b.txt(" "));

        if let Some(ref n) = self.body {
            let body_doc = n.build(b);
            result.push(body_doc);
        } else {
            result.push(b.txt(";"));
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FormalParameters {
    pub formal_parameters: Vec<FormalParameter>,
}

impl FormalParameters {
    pub fn new(node: Node) -> Self {
        let formal_parameters = node
            .try_cs_by_k("formal_parameter")
            .into_iter()
            .map(|n| FormalParameter::new(n))
            .collect();

        Self { formal_parameters }
    }
}

impl<'a> DocBuild<'a> for FormalParameters {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let modifiers_doc = b.to_docs(&self.formal_parameters);
        result.push(b.surround_choice(&modifiers_doc, ", ", ",", "(", ")"));
    }
}

#[derive(Debug, Serialize)]
pub struct FormalParameter {
    pub modifiers: Option<Modifiers>,
    pub type_: UnnanotatedType,
    pub name: String,
    //pub dimenssions
}

impl FormalParameter {
    pub fn new(node: Node) -> Self {
        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let type_ = UnnanotatedType::new(node.c_by_n("type"));
        let name = node.cvalue_by_n("name", source_code());

        Self {
            modifiers,
            type_,
            name,
        }
    }
}

impl<'a> DocBuild<'a> for FormalParameter {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.modifiers {
            result.push(n.build(b));
        }

        result.push(self.type_.build(b));
        result.push(b._txt(&self.name));
    }
}

#[derive(Debug, Serialize)]
pub struct SuperClass {
    pub type_: Type,
}

impl SuperClass {
    pub fn new(node: Node) -> Self {
        assert_check(node, "superclass");

        let type_ = Type::new(node.first_c());
        Self { type_ }
    }
}

impl<'a> DocBuild<'a> for SuperClass {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt(" extends "));
        result.push(self.type_.build(b));
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Modifiers {
    //pub buckets: CommentBuckets,
    annotation: Option<Annotation>,
    modifiers: Vec<Modifier>,
}

impl Modifiers {
    pub fn new(node: Node) -> Self {
        assert_check(node, "modifiers");
        let mut this = Self::default();

        for c in node.children_vec() {
            match c.kind() {
                "annotation" => {
                    this.annotation = Some(Annotation::new(c));
                }
                "modifier" => this.modifiers.push(Modifier::new(c.first_c())),
                "line_comment" | "block_comment" => continue,
                _ => panic!("## unknown node: {} in Modifiers", c.kind().red()),
            }
        }
        this
    }
}

impl<'a> DocBuild<'a> for Modifiers {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.annotation {
            result.push(n.build(b));
        }

        if !self.modifiers.is_empty() {
            let modifiers_doc = b.to_docs(&self.modifiers);
            result.push(b.intersperse_single_line(&modifiers_doc, " "));
            result.push(b.txt(" "));
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Annotation {
    pub name: String,
    pub arguments: Option<AnnotationArgumentList>,
}

impl Annotation {
    pub fn new(node: Node) -> Self {
        let name = node.cvalue_by_n("name", source_code());

        let mut arguments = None;
        node.try_c_by_n("arguments").map(|n| {
            arguments = Some(AnnotationArgumentList::new(n));
        });

        Self { name, arguments }
    }
}

impl<'a> DocBuild<'a> for Annotation {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt(format!("@{}", self.name)));

        if let Some(a) = &self.arguments {
            result.push(a.build(b));
        }
        result.push(b.nl());
    }
}

#[derive(Debug, Serialize)]
pub struct AnnotationKeyValue {
    key: String,
    value: String,
}

impl AnnotationKeyValue {
    pub fn new(node: Node) -> Self {
        assert_check(node, "annotation_key_value");
        Self {
            key: node.cvalue_by_n("key", source_code()),
            value: node.cvalue_by_n("value", source_code()),
        }
    }
}

impl<'a> DocBuild<'a> for AnnotationKeyValue {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt(&self.key));
        result.push(b.txt("="));
        result.push(b.txt(&self.value));
    }
}

#[derive(Debug, Serialize)]
pub struct ClassBody {
    pub class_members: Vec<FormattedMember<ClassMember>>,
}

impl ClassBody {
    pub fn new(node: Node) -> Self {
        assert_check(node, "class_body");
        let children = node.children_vec();
        let mut class_members: Vec<FormattedMember<ClassMember>> = Vec::new();

        for c in children {
            match c.kind() {
                "line_comment" | "block_comment" => continue,
                _ => {
                    let member = ClassMember::new(c);
                    let has_trailing_newlines = has_trailing_new_line(&c);
                    class_members.push(FormattedMember {
                        member,
                        has_trailing_newlines,
                    });
                }
            }
        }

        Self { class_members }
    }
}

impl<'a> DocBuild<'a> for ClassBody {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let body_doc = b.split_with_trailing_newline_considered(&self.class_members);
        result.push(body_doc);
    }
}

#[derive(Debug, Serialize)]
pub struct FieldDeclaration {
    pub buckets: Option<CommentBuckets>,
    pub modifiers: Option<Modifiers>,
    pub type_: UnnanotatedType,
    pub declarators: Vec<VariableDeclarator>,
    pub range: DataRange,
}

impl FieldDeclaration {
    pub fn new(node: Node) -> Self {
        assert_check(node, "field_declaration");
        let buckets = None;

        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));

        let type_node = node.c_by_n("type");
        let type_ = UnnanotatedType::new(type_node);

        let declarators = node
            .cs_by_n("declarator")
            .into_iter()
            .map(|n| VariableDeclarator::new(n))
            .collect();

        Self {
            buckets,
            modifiers,
            type_,
            declarators,
            range: DataRange::from(node.range()),
        }
    }
}

impl<'a> DocBuild<'a> for FieldDeclaration {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.modifiers {
            result.push(n.build(b));
        }

        result.push(self.type_.build(b));
        result.push(b.txt(" "));

        let decl_docs = b.to_docs(&self.declarators);
        let decision = b.group_elems_with_softline(&decl_docs, ",");
        result.push(decision);

        result.push(b.txt(";"));
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ArrayInitializer {
    variable_initializers: Vec<VariableInitializer>,
}

impl ArrayInitializer {
    pub fn new(node: Node, indent: usize) -> Self {
        assert_check(node, "array_initializer");
        ArrayInitializer::default()
    }
}

#[derive(Debug, Default, Serialize)]
pub struct AssignmentExpression {
    pub left: String,
    pub op: String,
    pub right: String,
}

impl AssignmentExpression {
    pub fn new(node: Node) -> Self {
        assert_check(node, "assignment_expression");

        let left = node.cvalue_by_n("left", source_code());
        let op = node.cvalue_by_n("operator", source_code());
        let right = node.cvalue_by_n("right", source_code());
        Self { left, op, right }
    }
}

impl<'a> DocBuild<'a> for AssignmentExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt(format!("{} {} {}", self.left, self.op, self.right)));
    }
}

//#[derive(Debug, Serialize)]
//pub struct Identifier {
//    pub value: String,
//}
//
//impl Identifier {
//    pub fn new(node: Node) -> Self {
//        Self {
//            value: node.value(source_code()),
//        }
//    }
//}

#[derive(Debug, Serialize)]
pub struct VoidType {
    pub value: String,
}

impl VoidType {
    pub fn new(node: Node) -> Self {
        Self {
            value: node.value(source_code()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DataRange {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_point: DataPoint,
    pub end_point: DataPoint,
}

impl From<Range> for DataRange {
    fn from(r: Range) -> Self {
        let start_point = DataPoint::from(r.start_point);
        let end_point = DataPoint::from(r.end_point);

        Self {
            start_byte: r.start_byte,
            end_byte: r.end_byte,
            start_point,
            end_point,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DataPoint {
    pub row: usize,
    pub column: usize,
}

impl From<Point> for DataPoint {
    fn from(p: Point) -> Self {
        Self {
            row: p.row,
            column: p.column,
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct CommentBuckets {
    pub pre_comments: Vec<Comment>,
    pub post_comments: Vec<Comment>,
}

#[derive(Debug, Serialize)]
pub struct Comment {
    pub id: usize,
    pub content: String,
    pub comment_type: CommentType,
    pub is_processed: bool,
    pub range: DataRange,
}

impl Comment {
    pub fn from_node(node: Node) -> Self {
        let id = node.id();
        let content = node.v(source_code()).to_string();
        Self {
            id,
            content,
            is_processed: false,
            comment_type: match node.kind() {
                "line_comment" => CommentType::Line,
                "block_comment" => CommentType::Block,
                _ => panic!("Unexpected comment type"),
            },
            range: DataRange::from(node.range()),
        }
    }
}

#[derive(Debug, Serialize)]
pub enum CommentType {
    Line,
    Block,
}

#[derive(Debug, Default, Serialize)]
pub struct Block {
    pub statements: Vec<Statement>,
}

impl Block {
    pub fn new(node: Node) -> Self {
        assert_check(node, "block");
        let mut this = Block::default();

        for c in node.children_vec() {
            this.statements.push(Statement::new(c));
        }
        this
    }
}

impl<'a> DocBuild<'a> for Block {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if self.statements.is_empty() {
            return result.push(b.concat(vec![b.txt("{"), b.nl(), b.txt("}")]));
        }

        let statement_docs = b.to_docs(&self.statements);
        let docs = b.surround_with_sep_and_softline(&statement_docs, "", "{", "}");
        result.push(docs);
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Interface {
    pub types: Vec<Type>,
}

impl Interface {
    pub fn new(node: Node) -> Self {
        assert_check(node, "interfaces");
        let mut interface = Interface::default();

        let type_list = node.c_by_k("type_list");

        for c in type_list.children_vec() {
            interface.types.push(Type::new(c));
        }

        interface
    }
}

impl<'a> DocBuild<'a> for Interface {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let types_doc = b.to_docs(&self.types);
        let implements_group = b.concat(vec![
            b.txt(" implements "),
            b.intersperse_single_line(&types_doc, ", "),
        ]);
        result.push(implements_group);
    }
}

#[derive(Debug, Serialize)]
pub struct MethodInvocation {
    pub object: Option<MethodObject>,
    pub property_navigation: Option<PropertyNavigation>,
    pub type_arguments: Option<TypeArguments>,
    pub name: String,
    pub arguments: ArgumentList,
}

impl MethodInvocation {
    pub fn new(node: Node) -> Self {
        let object = node.try_c_by_n("object").map(|n| {
            if n.kind() == "super" {
                MethodObject::Super(Super {})
            } else {
                MethodObject::Primary(Box::new(PrimaryExpression::new(n)))
            }
        });

        let property_navigation = object.as_ref().map(|_| {
            if node.try_c_by_n("safe_navigation_operator").is_some() {
                PropertyNavigation::SafeNavigationOperator
            } else {
                PropertyNavigation::Dot
            }
        });

        let type_arguments = node
            .try_c_by_k("type_arguments")
            .map(|n| TypeArguments::new(n));

        let name = node.cvalue_by_n("name", source_code());
        let arguments = ArgumentList::new(node.c_by_n("arguments"));

        Self {
            object,
            property_navigation,
            type_arguments,
            name,
            arguments,
        }
    }
}

impl<'a> DocBuild<'a> for MethodInvocation {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref o) = self.object {
            result.push(o.build(b));
        }

        if let Some(ref p) = self.property_navigation {
            result.push(p.build(b));
        }

        result.push(b.txt(&self.name));
        result.push(self.arguments.build(b));
    }
}

#[derive(Debug, Serialize)]
pub enum MethodObject {
    Super(Super),
    Primary(Box<PrimaryExpression>),
}

impl<'a> DocBuild<'a> for MethodObject {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            MethodObject::Super(s) => {
                result.push(s.build(b));
            }
            MethodObject::Primary(p) => {
                result.push(p.build(b));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TypeArguments {
    pub types: Vec<Type>,
}

impl TypeArguments {
    pub fn new(node: Node) -> Self {
        let mut types = Vec::new();
        for c in node.children_vec() {
            types.push(Type::new(c));
        }
        Self { types }
    }
}

impl<'a> DocBuild<'a> for TypeArguments {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let types_doc = b.to_docs(&self.types);
        result.push(b.surround_choice(&types_doc, ", ", ",", "<", ">"));
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ArgumentList {
    pub expressions: Vec<Expression>,
}

impl ArgumentList {
    pub fn new(node: Node) -> Self {
        let mut this = ArgumentList::default();
        for c in node.children_vec() {
            this.expressions.push(Expression::new(c));
        }
        this
    }
}

impl<'a> DocBuild<'a> for ArgumentList {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let exp_doc = b.to_docs(&self.expressions);
        result.push(b.surround_choice(&exp_doc, ", ", ",", "(", ")"));
    }
}

#[derive(Debug, Serialize)]
pub struct Super {}

impl<'a> DocBuild<'a> for Super {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("super"))
    }
}

#[derive(Debug, Serialize)]
pub struct This {}

impl<'a> DocBuild<'a> for This {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("this"))
    }
}

#[derive(Debug, Serialize)]
pub struct BinaryExpression {
    pub left: Expression,
    pub op: String,
    pub right: Expression,
}

impl BinaryExpression {
    pub fn new(node: Node) -> Self {
        let left = Expression::new(node.c_by_n("left"));
        let op = node.cvalue_by_n("operator", source_code());
        let right = Expression::new(node.c_by_n("right"));

        Self { left, op, right }
    }
}

impl<'a> DocBuild<'a> for BinaryExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let docs_vec = b.to_docs(vec![&self.left, &self.right]);
        let sep = format!(" {}", &self.op);
        let decision = b.group_elems_with_softline(&docs_vec, &sep);
        result.push(decision);
    }
}

#[derive(Debug, Serialize)]
pub struct LocalVariableDeclaration {
    pub modifiers: Option<Modifiers>,
    pub type_: UnnanotatedType,
    pub declarators: Vec<VariableDeclarator>,
}

impl LocalVariableDeclaration {
    pub fn new(node: Node) -> Self {
        assert_check(node, "local_variable_declaration");

        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let type_ = UnnanotatedType::new(node.c_by_n("type"));
        let declarators = node
            .cs_by_n("declarator")
            .into_iter()
            .map(|n| VariableDeclarator::new(n))
            .collect();

        Self {
            modifiers,
            type_,
            declarators,
        }
    }
}

impl<'a> DocBuild<'a> for LocalVariableDeclaration {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.modifiers {
            result.push(n.build(b));
        }

        result.push(self.type_.build(b));
        result.push(b.txt(" "));

        let docs_vec = b.to_docs(&self.declarators);
        let decision = b.group_elems_with_softline(&docs_vec, ",");
        result.push(decision);
    }
}

#[derive(Debug, Serialize)]
pub struct VariableDeclarator {
    pub name: String,
    //pub dimenssions
    pub value: Option<VariableInitializer>,
}

impl VariableDeclarator {
    pub fn new(node: Node) -> Self {
        assert_check(node, "variable_declarator");
        let name = node.cvalue_by_n("name", source_code());

        let value = node.try_c_by_n("value").map(|n| match n.kind() {
            //"array_initializer" => {
            //    VariableInitializer::ArrayInitializer(ArrayInitializer::new(v, source_code, indent))
            //}
            //_ => VariableInitializer::Expression(Expression::Primary(Box::new(
            //    PrimaryExpression::Identifier(v.value(source_code())),
            //))),
            _ => VariableInitializer::Expression(Expression::new(n)),
        });

        Self { name, value }
    }
}

impl<'a> DocBuild<'a> for VariableDeclarator {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt(&self.name));
        if let Some(ref v) = self.value {
            result.push(b.txt(" = "));
            result.push(v.build(b));
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GenericType {
    pub generic_identifier: GenericIdentifier,
    pub type_arguments: TypeArguments,
}

impl GenericType {
    pub fn new(node: Node) -> Self {
        assert_check(node, "generic_type");

        let generic_identifier = if let Some(t) = node.try_c_by_k("type_identifier") {
            GenericIdentifier::Type(t.value(source_code()))
        } else if let Some(s) = node.try_c_by_k("scoped_type_identifier") {
            GenericIdentifier::Scoped(ScopedTypeIdentifier::new(s))
        } else {
            panic!("## can't build generic_identifier node in GenericType");
        };

        let type_arguments = TypeArguments::new(node.c_by_k("type_arguments"));

        Self {
            generic_identifier,
            type_arguments,
        }
    }
}

impl<'a> DocBuild<'a> for GenericType {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(self.generic_identifier.build(b));
        result.push(self.type_arguments.build(b));
    }
}

#[derive(Debug, Serialize)]
pub enum GenericIdentifier {
    Type(String),
    Scoped(ScopedTypeIdentifier),
}

impl<'a> DocBuild<'a> for GenericIdentifier {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Type(s) => {
                result.push(b.txt(s));
            }
            Self::Scoped(s) => {
                result.push(s.build(b));
            }
        }
    }
}
#[derive(Debug, Serialize)]
pub struct IfStatement {
    pub condition: ParenthesizedExpression,
    pub consequence: Statement,
    pub alternative: Option<Statement>,
}

impl IfStatement {
    pub fn new(node: Node) -> Self {
        let condition = ParenthesizedExpression::new(node.c_by_n("condition"));
        let consequence = Statement::new(node.c_by_n("consequence"));
        let alternative = node.try_c_by_n("alternative").map(|a| Statement::new(a));
        Self {
            condition,
            consequence,
            alternative,
        }
    }
}

impl<'a> DocBuild<'a> for IfStatement {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("if "));
        result.push(self.condition.build(b));

        if self.consequence.is_block() {
            result.push(b.txt(" "));
            result.push(self.consequence.build(b));
        } else {
            result.push(b.add_indent_level(b.nl()));
            result.push(b.add_indent_level(self.consequence.build(b)));
        }

        // Handle the 'else' part
        if let Some(ref a) = self.alternative {
            match a {
                Statement::If(_) => {
                    if self.consequence.is_block() {
                        result.push(b.txt(" else "));
                    } else {
                        result.push(b.nl());
                        result.push(b.txt("else "));
                    }
                    result.push(a.build(b)); // Recursively build the nested 'else if' statement
                }
                Statement::Block(_) => {
                    if self.consequence.is_block() {
                        result.push(b.txt(" else "));
                    } else {
                        result.push(b.nl());
                        result.push(b.txt("else "));
                    }
                    result.push(a.build(b));
                }
                // Handle "else" with a single statement
                _ => {
                    if self.consequence.is_block() {
                        result.push(b.txt(" else "));
                    } else {
                        result.push(b.nl());
                        result.push(b.txt("else"));
                        result.push(b.add_indent_level(b.nl()));
                    }
                    result.push(a.build(b)); // Build the else statement
                }
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ParenthesizedExpression {
    pub exp: Expression,
}

impl ParenthesizedExpression {
    pub fn new(node: Node) -> Self {
        let exp = Expression::new(node.first_c());
        Self { exp }
    }
}

impl<'a> DocBuild<'a> for ParenthesizedExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("("));
        result.push(self.exp.build(b));
        result.push(b.txt(")"));
    }
}

#[derive(Debug, Serialize)]
pub struct ForStatement {
    pub init: Option<LocalVariableDeclaration>,
    pub condition: Option<Expression>,
    pub update: Option<Expression>,
    pub body: Statement,
}

impl ForStatement {
    pub fn new(node: Node) -> Self {
        let init = node
            .try_c_by_n("init")
            .map(|n| LocalVariableDeclaration::new(n));
        let condition = node.try_c_by_n("condition").map(|n| Expression::new(n));
        let update = node.try_c_by_n("update").map(|n| Expression::new(n));
        let body = Statement::new(node.c_by_n("body"));
        Self {
            init,
            condition,
            update,
            body,
        }
    }
}

impl<'a> DocBuild<'a> for ForStatement {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("for "));
        let init = match &self.init {
            Some(i) => i.build(b),
            None => b.nil(),
        };
        let condition = match &self.condition {
            Some(c) => b.concat(vec![b.txt(" "), c.build(b)]),
            None => b.nil(),
        };
        let update = match &self.update {
            Some(u) => b.concat(vec![b.txt(" "), u.build(b)]),
            None => b.nil(),
        };
        let docs = vec![init, condition, update];

        result.push(b.surround_choice(&docs, ";", ";", "(", ")"));
        result.push(b.txt(" "));
        result.push(self.body.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct EnhancedForStatement {
    pub modifiers: Option<Modifiers>,
    pub type_: UnnanotatedType,
    pub name: String,
    //pub dimension
    pub value: Expression,
    pub body: Statement,
}

impl EnhancedForStatement {
    pub fn new(node: Node) -> Self {
        assert_check(node, "enhanced_for_statement");

        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let type_ = UnnanotatedType::new(node.c_by_n("type"));
        let name = node.cvalue_by_n("name", source_code());
        let value = Expression::new(node.c_by_n("value"));
        let body = Statement::new(node.c_by_n("body"));
        Self {
            modifiers,
            type_,
            name,
            value,
            body,
        }
    }
}

impl<'a> DocBuild<'a> for EnhancedForStatement {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("for ("));
        result.push(self.type_.build(b));
        result.push(b._txt(&self.name));
        result.push(b._txt_(":"));
        result.push(self.value.build(b));
        result.push(b.txt_(")"));
        result.push(self.body.build(b));
    }
}
#[derive(Debug, Serialize)]
pub enum UpdateExpression {
    Pre {
        operator: String,
        operand: Box<Expression>,
    },
    Post {
        operand: Box<Expression>,
        operator: String,
    },
}

impl UpdateExpression {
    pub fn new(node: Node) -> Self {
        assert_check(node, "update_expression");

        let operator_node = node.c_by_n("operator");
        let operand_node = node.c_by_n("operand");

        if operator_node.start_byte() < operand_node.start_byte() {
            Self::Pre {
                operator: operator_node.value(source_code()),
                operand: Box::new(Expression::new(operand_node)),
            }
        } else {
            Self::Post {
                operand: Box::new(Expression::new(operand_node)),
                operator: operator_node.value(source_code()),
            }
        }
    }
}

impl<'a> DocBuild<'a> for UpdateExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Pre { operator, operand } => {
                result.push(b.txt(operator));
                result.push(operand.build(b));
            }
            Self::Post { operand, operator } => {
                result.push(operand.build(b));
                result.push(b.txt(operator));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ScopedTypeIdentifier {
    pub scoped_choice: ScopedChoice,
    pub annotations: Vec<Annotation>,
    pub type_identifier: String,
}

impl ScopedTypeIdentifier {
    pub fn new(node: Node) -> Self {
        assert_check(node, "scoped_type_identifier");

        let prefix_node = node.first_c();
        let scoped_choice = match prefix_node.kind() {
            "type_identifier" => ScopedChoice::TypeIdentifier(prefix_node.value(source_code())),
            "scoped_type_identifier" => ScopedChoice::Scoped(Box::new(Self::new(prefix_node))),
            "generic_type" => ScopedChoice::Generic(Box::new(GenericType::new(prefix_node))),
            _ => panic!(
                "## unknown node: {} in ScopedTypeIdentifier prefix node",
                prefix_node.kind().red()
            ),
        };

        let annotations: Vec<_> = node
            .try_cs_by_k("annotation")
            .into_iter()
            .map(|n| Annotation::new(n))
            .collect();

        let type_identifier_node = node
            .cs_by_k("type_identifier")
            .pop()
            .expect("## mandatory node type_identifier missing in ScopedTypeIdentifier");
        let type_identifier = type_identifier_node.value(source_code());

        Self {
            scoped_choice,
            annotations,
            type_identifier,
        }
    }
}

impl<'a> DocBuild<'a> for ScopedTypeIdentifier {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(self.scoped_choice.build(b));
        result.push(b.txt("."));
        if !self.annotations.is_empty() {
            let docs = b.to_docs(&self.annotations);
            result.push(b.intersperse_single_line(&docs, " "));
            result.push(b.txt(" "));
        }
        result.push(b.txt(&self.type_identifier));
    }
}

#[derive(Debug, Serialize)]
pub enum ScopedChoice {
    TypeIdentifier(String),
    Scoped(Box<ScopedTypeIdentifier>),
    Generic(Box<GenericType>),
}

impl<'a> DocBuild<'a> for ScopedChoice {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::TypeIdentifier(t) => {
                result.push(b.txt(t));
            }
            Self::Scoped(s) => {
                result.push(s.build(b));
            }
            Self::Generic(g) => {
                result.push(g.build(b));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConstructorDeclaration {
    pub modifiers: Option<Modifiers>,
    pub type_parameters: Option<TypeParameters>,
    pub name: String,
    pub parameters: FormalParameters,
    pub body: ConstructorBody,
}

impl ConstructorDeclaration {
    pub fn new(node: Node) -> Self {
        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let type_parameters = node
            .try_c_by_k("type_parameters")
            .map(|n| TypeParameters::new(n));
        let name = node.cvalue_by_n("name", source_code());
        let parameters = FormalParameters::new(node.c_by_n("parameters"));
        let body = ConstructorBody::new(node.c_by_n("body"));
        Self {
            modifiers,
            type_parameters,
            name,
            parameters,
            body,
        }
    }
}

impl<'a> DocBuild<'a> for ConstructorDeclaration {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.modifiers {
            result.push(n.build(b));
        }
        if let Some(ref n) = self.type_parameters {
            result.push(n.build(b));
        }

        result.push(b.txt(&self.name));
        result.push(self.parameters.build(b));
        result.push(b.txt(" "));
        result.push(self.body.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct ConstructorBody {
    pub constructor_invocation: Option<ConstructInvocation>,
    pub statements: Vec<Statement>,
}

impl ConstructorBody {
    pub fn new(node: Node) -> Self {
        let mut constructor_invocation = None;
        let mut statements = Vec::new();

        for (i, c) in node.children_vec().into_iter().enumerate() {
            if i == 0 && c.kind() == "explicit_constructor_invocation" {
                constructor_invocation = Some(ConstructInvocation::new(c));
            }
            statements.push(Statement::new(c));
        }

        ConstructorBody {
            constructor_invocation,
            statements,
        }
    }
}

impl<'a> DocBuild<'a> for ConstructorBody {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let member_docs = b.to_docs(&self.statements);
        let statements_doc = b.intersperse_with_sep_and_softline(&member_docs, "");
        result.push(statements_doc);

        if self.constructor_invocation.is_none() && self.statements.is_empty() {
            return result.push(b.concat(vec![b.txt("{"), b.nl(), b.txt("}")]));
        }

        let mut doc_vec = Vec::new();
        if let Some(ref c) = self.constructor_invocation {
            doc_vec.push(c.build(b));
        }

        self.statements
            .iter()
            .for_each(|n| doc_vec.push(n.build(b)));

        let docs = b.surround_choice(&doc_vec, "", "", "{", "}");
        result.push(docs);
    }
}

#[derive(Debug, Serialize)]
pub struct ConstructInvocation {
    pub object: Option<Box<PrimaryExpression>>,
    pub type_arguments: Option<TypeArguments>,
    pub constructor: Constructor,
}

impl ConstructInvocation {
    pub fn new(node: Node) -> Self {
        let object = node
            .try_c_by_n("object")
            .map(|n| Box::new(PrimaryExpression::new(n)));

        let type_arguments = node
            .try_c_by_k("type_arguments")
            .map(|n| TypeArguments::new(n));

        let constructor = match node.c_by_n("constructor").kind() {
            "this" => Constructor::This,
            "super" => Constructor::Super,
            other => panic!("## unknown node: {} in Constructor", other.red()),
        };

        Self {
            object,
            type_arguments,
            constructor,
        }
    }
}

impl<'a> DocBuild<'a> for ConstructInvocation {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref o) = self.object {
            result.push(o.build(b));
        }

        if let Some(ref t) = self.type_arguments {
            result.push(t.build(b));
        }

        result.push(self.constructor.build(b));
    }
}

#[derive(Debug, Serialize)]
enum Constructor {
    This,
    Super,
}

impl<'a> DocBuild<'a> for Constructor {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::This => result.push(b.txt("this")),
            Self::Super => result.push(b.txt("super")),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TypeParameters {
    pub type_parameters: Vec<TypeParameter>,
}

impl TypeParameters {
    pub fn new(node: Node) -> Self {
        let type_parameters: Vec<_> = node
            .cs_by_k("type_parameter")
            .into_iter()
            .map(|n| TypeParameter::new(n))
            .collect();
        Self { type_parameters }
    }
}

impl<'a> DocBuild<'a> for TypeParameters {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        self.type_parameters
            .iter()
            .for_each(|t| result.push(t.build(b)));
    }
}

#[derive(Debug, Serialize)]
pub struct TypeParameter {
    annotations: Vec<Annotation>,
    pub type_identifier: String,
}

impl TypeParameter {
    pub fn new(node: Node) -> Self {
        let annotations: Vec<_> = node
            .try_cs_by_k("annotation")
            .into_iter()
            .map(|n| Annotation::new(n))
            .collect();

        let type_identifier = node.cvalue_by_k("type_identifier", source_code());
        Self {
            annotations,
            type_identifier,
        }
    }
}

impl<'a> DocBuild<'a> for TypeParameter {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt(&self.type_identifier));
    }
}

#[derive(Debug, Serialize)]
pub struct ObjectCreationExpression {
    pub type_arguments: Option<TypeArguments>,
    pub type_: UnnanotatedType,
    pub arguments: AnnotationArgumentList,
    pub class_body: ClassBody,
}

impl ObjectCreationExpression {
    pub fn new(node: Node) -> Self {}
}

impl<'a> DocBuild<'a> for ObjectCreationExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {}
}
