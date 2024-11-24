use crate::{
    accessor::Accessor,
    doc::DocRef,
    doc_builder::{DocBuilder, Insertable},
    enum_def::*,
    utility::{assert_check, has_trailing_new_line, source_code},
};
use colored::Colorize;
use serde::Serialize;
use std::{collections::HashSet, fmt::Debug};
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
                "enum_declaration" => root
                    .members
                    .push(RootMember::Enum(Box::new(EnumDeclaration::new(c)))),
                "trigger_declaration" => root
                    .members
                    .push(RootMember::Trigger(Box::new(TriggerDeclaration::new(c)))),
                "interface_declaration" => {
                    root.members
                        .push(RootMember::Interface(Box::new(InterfaceDeclaration::new(
                            c,
                        ))))
                }
                _ => panic!("## unknown node: {} in Root ", c.kind().red()),
            }
        }
        root
    }
}

impl<'a> DocBuild<'a> for Root {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let docs = b.to_docs(&self.members);
        let sep = Insertable::new::<&str>(None, None, Some(b.nl()));
        let doc = b.intersperse(&docs, sep);
        result.push(doc);
        result.push(b.nl());
    }
}

#[derive(Debug, Serialize)]
pub struct ClassDeclaration {
    pub buckets: Option<CommentBuckets>,
    pub modifiers: Option<Modifiers>,
    pub name: String,
    pub type_parameters: Option<TypeParameters>,
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
        let type_parameters = node
            .try_c_by_k("type_parameters")
            .map(|n| TypeParameters::new(n));
        let superclass = node.try_c_by_k("superclass").map(|n| SuperClass::new(n));
        let interface = node.try_c_by_k("interfaces").map(|n| Interface::new(n));
        let body = ClassBody::new(node.c_by_n("body"));
        let range = DataRange::from(node.range());

        Self {
            buckets,
            modifiers,
            name,
            type_parameters,
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

        if let Some(ref n) = self.type_parameters {
            result.push(n.build(b));
        }
        if let Some(ref n) = self.superclass {
            result.push(n.build(b));
        }

        if let Some(ref n) = self.interface {
            result.push(n.build(b));
        }

        result.push(b.txt(" "));
        result.push(self.body.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct MethodDeclaration {
    pub modifiers: Option<Modifiers>,
    pub type_: UnannotatedType,
    pub name: String,
    pub formal_parameters: FormalParameters,
    pub body: Option<Block>,
    //pub dimentions
}

impl MethodDeclaration {
    pub fn new(node: Node) -> Self {
        assert_check(node, "method_declaration");

        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let type_ = UnannotatedType::new(node.c_by_n("type"));
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

        if let Some(ref n) = self.body {
            result.push(b.txt(" "));
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

        let sep = Insertable::new(None, Some(","), Some(b.softline()));
        let open = Insertable::new(None, Some("("), Some(b.maybeline()));
        let close = Insertable::new(Some(b.maybeline()), Some(")"), None);
        let doc = b.group(b.surround(&modifiers_doc, sep, open, close));
        result.push(doc);
    }
}

#[derive(Debug, Serialize)]
pub struct FormalParameter {
    pub modifiers: Option<Modifiers>,
    pub type_: UnannotatedType,
    pub name: String,
    //pub dimenssions
}

impl FormalParameter {
    pub fn new(node: Node) -> Self {
        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let type_ = UnannotatedType::new(node.c_by_n("type"));
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
            let docs = b.to_docs(&self.modifiers);
            let sep = Insertable::new(None, Some(" "), None);
            result.push(b.intersperse(&docs, sep));
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
    pub class_members: Vec<BodyMember<ClassMember>>,
}

impl ClassBody {
    pub fn new(node: Node) -> Self {
        assert_check(node, "class_body");

        let class_members: Vec<_> = node
            .children_vec()
            .into_iter()
            .map(|n| BodyMember {
                member: ClassMember::new(n),
                has_trailing_newline: has_trailing_new_line(&n),
            })
            .collect();

        Self { class_members }
    }
}

impl<'a> DocBuild<'a> for ClassBody {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.surround_body(&self.class_members, "{", "}"));
    }
}

#[derive(Debug, Serialize)]
pub struct FieldDeclaration {
    pub buckets: Option<CommentBuckets>,
    pub modifiers: Option<Modifiers>,
    pub type_: UnannotatedType,
    pub declarators: Vec<VariableDeclarator>,
    pub accessor_list: Option<AccessorList>,
    pub range: DataRange,
}

impl FieldDeclaration {
    pub fn new(node: Node) -> Self {
        assert_check(node, "field_declaration");
        let buckets = None;

        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));

        let type_node = node.c_by_n("type");
        let type_ = UnannotatedType::new(type_node);

        let declarators = node
            .cs_by_n("declarator")
            .into_iter()
            .map(|n| VariableDeclarator::new(n))
            .collect();

        let accessor_list = node
            .try_c_by_k("accessor_list")
            .map(|n| AccessorList::new(n));

        Self {
            buckets,
            modifiers,
            type_,
            declarators,
            accessor_list,
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
        let sep = Insertable::new(None, Some(","), Some(b.softline()));
        let doc = b.group_then_indent(b.intersperse(&decl_docs, sep));
        result.push(doc);

        if let Some(ref n) = self.accessor_list {
            result.push(b.txt(" "));
            result.push(n.build(b));
        } else {
            result.push(b.txt(";"));
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ArrayInitializer {
    initializers: Vec<VariableInitializer>,
}

impl ArrayInitializer {
    pub fn new(node: Node) -> Self {
        assert_check(node, "array_initializer");

        let initializers: Vec<_> = node
            .children_vec()
            .into_iter()
            .map(|n| VariableInitializer::new(n))
            .collect();

        Self { initializers }
    }
}

impl<'a> DocBuild<'a> for ArrayInitializer {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let docs = b.to_docs(&self.initializers);

        let sep = Insertable::new(None, Some(","), Some(b.softline()));
        let open = Insertable::new(None, Some("{"), Some(b.softline()));
        let close = Insertable::new(Some(b.softline()), Some("}"), None);
        let doc = b.group(b.surround(&docs, sep, open, close));
        result.push(doc);
    }
}

#[derive(Debug, Serialize)]
pub struct AssignmentExpression {
    pub left: AssignmentLeft,
    pub op: String,
    pub right: Expression,
}

impl AssignmentExpression {
    pub fn new(node: Node) -> Self {
        assert_check(node, "assignment_expression");

        let left = AssignmentLeft::new(node.c_by_n("left"));
        let op = node.cvalue_by_n("operator", source_code());
        let right = Expression::new(node.c_by_n("right"));
        Self { left, op, right }
    }
}

impl<'a> DocBuild<'a> for AssignmentExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(self.left.build(b));
        result.push(b._txt_(&self.op));
        result.push(self.right.build(b));
    }
}

#[derive(Debug, Serialize)]
pub enum AssignmentLeft {
    Identifier(String),
    Field(FieldAccess),
    Array(ArrayAccess),
}

impl AssignmentLeft {
    pub fn new(n: Node) -> Self {
        match n.kind() {
            "identifier" => Self::Identifier(n.value(source_code())),
            "field_access" => Self::Field(FieldAccess::new(n)),
            "array_access" => Self::Array(ArrayAccess::new(n)),
            _ => panic!("## unknown node: {} in AssignmentLeft", n.kind().red()),
        }
    }
}

impl<'a> DocBuild<'a> for AssignmentLeft {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Identifier(n) => {
                result.push(b.txt(&n));
            }
            Self::Field(n) => {
                result.push(n.build(b));
            }
            Self::Array(n) => {
                result.push(n.build(b));
            }
        }
    }
}

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

#[derive(Debug, Serialize)]
pub struct Block {
    pub statements: Vec<BodyMember<Statement>>,
}

impl Block {
    pub fn new(node: Node) -> Self {
        assert_check(node, "block");

        let statements: Vec<BodyMember<Statement>> = node
            .children_vec()
            .into_iter()
            .map(|n| BodyMember {
                member: Statement::new(n),
                has_trailing_newline: has_trailing_new_line(&n),
            })
            .collect();

        Self { statements }
    }
}

impl<'a> DocBuild<'a> for Block {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if self.statements.is_empty() {
            return result.push(b.concat(vec![b.txt("{"), b.nl(), b.txt("}")]));
        }

        let docs = b.surround_body(&self.statements, "{", "}");
        result.push(docs);
    }
}

#[derive(Debug, Serialize)]
pub struct Interface {
    pub types: Vec<Type>,
}

impl Interface {
    pub fn new(node: Node) -> Self {
        assert_check(node, "interfaces");

        let types = node
            .c_by_k("type_list")
            .children_vec()
            .into_iter()
            .map(|n| Type::new(n))
            .collect();

        Self { types }
    }
}

impl<'a> DocBuild<'a> for Interface {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let docs = b.to_docs(&self.types);
        let sep = Insertable::new(None, Some(", "), None);
        let doc = b.intersperse(&docs, sep);

        let implements_group = b.concat(vec![b._txt_("implements"), doc]);
        result.push(implements_group);
    }
}

#[derive(Debug, Serialize)]
pub struct MethodInvocation {
    pub is_base: bool, // whehter it's the top layer or the nested;
    pub object: Option<MethodObject>,
    pub property_navigation: Option<PropertyNavigation>,
    pub type_arguments: Option<TypeArguments>,
    pub name: String,
    pub arguments: ArgumentList,
}

impl MethodInvocation {
    pub fn new(node: Node) -> Self {
        assert_check(node, "method_invocation");

        let is_base = node.parent().unwrap().kind() != "method_invocation";

        let object = node.try_c_by_n("object").map(|n| {
            if n.kind() == "super" {
                MethodObject::Super(Super {})
            } else {
                MethodObject::Primary(Box::new(PrimaryExpression::new(n)))
            }
        });

        let property_navigation = object.as_ref().map(|_| {
            if node.try_c_by_k("safe_navigaion_operator").is_some() {
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
            is_base,
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
        let mut docs = vec![];
        if let Some(ref o) = self.object {
            docs.push(o.build(b));
            docs.push(b.maybeline());
        }

        if let Some(ref p) = self.property_navigation {
            docs.push(p.build(b));
        }

        docs.push(b.txt(&self.name));
        docs.push(self.arguments.build(b));

        let mut doc = b.concat(docs);

        if self.is_base {
            doc = b.group_then_indent(doc);
        }
        result.push(doc);
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
        let docs = b.to_docs(&self.types);

        let sep = Insertable::new(None, Some(","), Some(b.softline()));
        let open = Insertable::new(None, Some("<"), Some(b.maybeline()));
        let close = Insertable::new(Some(b.maybeline()), Some(">"), None);
        let doc = b.group(b.surround(&docs, sep, open, close));
        result.push(doc);
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ArgumentList {
    pub expressions: Vec<Expression>,
}

impl ArgumentList {
    pub fn new(node: Node) -> Self {
        let expressions = node
            .children_vec()
            .into_iter()
            .map(|n| Expression::new(n))
            .collect();
        Self { expressions }
    }
}

impl<'a> DocBuild<'a> for ArgumentList {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let docs = b.to_docs(&self.expressions);

        let sep = Insertable::new(None, Some(","), Some(b.softline()));
        let open = Insertable::new(None, Some("("), Some(b.maybeline()));
        let close = Insertable::new(Some(b.maybeline()), Some(")"), None);
        let doc = b.group(b.surround(&docs, sep, open, close));
        result.push(doc);
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

impl This {
    pub fn new(_: Node) -> Self {
        Self {}
    }
}

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
        let op = node.c_by_n("operator").kind().to_string();
        let right = Expression::new(node.c_by_n("right"));
        Self { left, op, right }
    }
}

impl<'a> DocBuild<'a> for BinaryExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let docs = b.to_docs(vec![&self.left, &self.right]);
        let sep = Insertable::new(None, Some(format!(" {}", &self.op)), Some(b.softline()));
        let doc = b.group_then_indent(b.intersperse(&docs, sep));
        result.push(doc);
    }
}

#[derive(Debug, Serialize)]
pub struct LocalVariableDeclaration {
    pub modifiers: Option<Modifiers>,
    pub type_: UnannotatedType,
    pub declarators: Vec<VariableDeclarator>,
}

impl LocalVariableDeclaration {
    pub fn new(node: Node) -> Self {
        assert_check(node, "local_variable_declaration");

        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let type_ = UnannotatedType::new(node.c_by_n("type"));
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

        let docs = b.to_docs(&self.declarators);

        // prevent unnessary indentation when only one element;
        let doc = if docs.len() == 1 {
            docs[0]
        } else {
            let sep = Insertable::new(None, Some(","), Some(b.softline()));
            b.group_then_indent(b.intersperse(&docs, sep))
        };

        result.push(doc);
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
            _ => VariableInitializer::Exp(Expression::new(n)),
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
            result.push(b.indent(b.nl()));
            result.push(b.indent(self.consequence.build(b)));
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
                        result.push(b.indent(b.nl()));
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
        // to align with prettier apex
        result.push(b.txt("("));
        let doc = b.concat(vec![
            b.indent(b.maybeline()),
            b.indent(self.exp.build(b)),
            b.maybeline(),
        ]);
        result.push(b.group(doc));
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

        let sep = Insertable::new(None, Some(";"), Some(b.maybeline()));
        let open = Insertable::new(None, Some("("), Some(b.maybeline()));
        let close = Insertable::new(Some(b.maybeline()), Some(")"), None);
        let doc = b.group(b.surround(&docs, sep, open, close));
        result.push(doc);

        match self.body {
            Statement::SemiColumn => result.push(b.txt(";")),
            _ => {
                result.push(b.txt(" "));
                result.push(self.body.build(b));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EnhancedForStatement {
    pub modifiers: Option<Modifiers>,
    pub type_: UnannotatedType,
    pub name: String,
    //pub dimension
    pub value: Expression,
    pub body: Statement,
}

impl EnhancedForStatement {
    pub fn new(node: Node) -> Self {
        assert_check(node, "enhanced_for_statement");

        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let type_ = UnannotatedType::new(node.c_by_n("type"));
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
        result.push(b.txt(")"));
        match self.body {
            Statement::SemiColumn => result.push(b.txt(";")),
            _ => {
                result.push(b.txt(" "));
                result.push(self.body.build(b));
            }
        }
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

            let sep = Insertable::new(None, Some(" "), None);
            result.push(b.intersperse(&docs, sep));
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
    pub constructor_invocation: Option<BodyMember<ConstructInvocation>>,
    pub statements: Vec<BodyMember<Statement>>,
}

impl ConstructorBody {
    pub fn new(node: Node) -> Self {
        let mut constructor_invocation = None;
        let mut statements: Vec<BodyMember<Statement>> = Vec::new();

        for (i, c) in node.children_vec().into_iter().enumerate() {
            if i == 0 && c.kind() == "explicit_constructor_invocation" {
                let member = ConstructInvocation::new(c);
                let has_trailing_newline = has_trailing_new_line(&c);
                constructor_invocation = Some(BodyMember {
                    member,
                    has_trailing_newline,
                });
            } else {
                let member = Statement::new(c);
                let has_trailing_newline = has_trailing_new_line(&c);
                statements.push(BodyMember {
                    member,
                    has_trailing_newline,
                });
            }
        }

        Self {
            constructor_invocation,
            statements,
        }
    }
}

impl<'a> DocBuild<'a> for ConstructorBody {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if self.constructor_invocation.is_none() && self.statements.is_empty() {
            return result.push(b.concat(vec![b.txt("{"), b.nl(), b.txt("}")]));
        }

        result.push(b.txt("{"));
        result.push(b.indent(b.nl()));

        if let Some(c) = &self.constructor_invocation {
            result.push(c.member.build(b));
            result.push(b.txt(";"));

            if !self.statements.is_empty() {
                if c.has_trailing_newline {
                    result.push(b.nl_with_no_indent());
                }
                result.push(b.nl());
            }
        }
        result.push(b.indent(b.intersperse_body_members(&self.statements)));

        result.push(b.nl());
        result.push(b.txt("}"));
    }
}

#[derive(Debug, Serialize)]
pub struct ConstructInvocation {
    pub object: Option<Box<PrimaryExpression>>,
    pub type_arguments: Option<TypeArguments>,
    pub constructor: Option<Constructor>,
    pub arguments: ArgumentList,
}

impl ConstructInvocation {
    pub fn new(node: Node) -> Self {
        let object = node
            .try_c_by_n("object")
            .map(|n| Box::new(PrimaryExpression::new(n)));

        let type_arguments = node
            .try_c_by_k("type_arguments")
            .map(|n| TypeArguments::new(n));

        let constructor = node.try_c_by_n("constructor").map(|n| match n.kind() {
            "this" => Constructor::This,
            "super" => Constructor::Super,
            other => panic!("## unknown node: {} in Constructor", other.red()),
        });

        let arguments = ArgumentList::new(node.c_by_n("arguments"));

        Self {
            object,
            type_arguments,
            constructor,
            arguments,
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

        if let Some(ref c) = self.constructor {
            result.push(c.build(b));
        }

        result.push(self.arguments.build(b));
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
        let docs = b.to_docs(&self.type_parameters);

        let sep = Insertable::new(None, Some(","), Some(b.softline()));
        let open = Insertable::new(None, Some("<"), Some(b.maybeline()));
        let close = Insertable::new(Some(b.maybeline()), Some(">"), None);
        let doc = b.group(b.surround(&docs, sep, open, close));
        result.push(doc);
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
    pub type_: UnannotatedType,
    pub arguments: ArgumentList,
    pub class_body: Option<ClassBody>,
}

impl ObjectCreationExpression {
    pub fn new(node: Node) -> Self {
        assert_check(node, "object_creation_expression");

        let type_arguments = node
            .try_c_by_k("type_arguments")
            .map(|n| TypeArguments::new(n));

        let type_ = UnannotatedType::new(node.c_by_n("type"));
        let arguments = ArgumentList::new(node.c_by_n("arguments"));
        let class_body = node.try_c_by_k("class_body").map(|n| ClassBody::new(n));

        Self {
            type_arguments,
            type_,
            arguments,
            class_body,
        }
    }
}

impl<'a> DocBuild<'a> for ObjectCreationExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("new"));
        if let Some(t) = &self.type_arguments {
            result.push(t.build(b));
        }

        result.push(b.txt(" "));
        result.push(self.type_.build(b));
        result.push(self.arguments.build(b));

        if let Some(c) = &self.class_body {
            result.push(c.build(b));
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RunAsStatement {
    pub user: ParenthesizedExpression,
    pub block: Block,
}

impl RunAsStatement {
    pub fn new(node: Node) -> Self {
        assert_check(node, "run_as_statement");

        let user = ParenthesizedExpression::new(node.c_by_n("user"));
        let block = Block::new(node.c_by_k("block"));
        Self { user, block }
    }
}

impl<'a> DocBuild<'a> for RunAsStatement {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("System.runAs"));
        result.push(self.user.build(b));
        result.push(b.txt(" "));
        result.push(self.block.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct DoStatement {
    pub body: Block,
    pub condition: ParenthesizedExpression,
}

impl DoStatement {
    pub fn new(node: Node) -> Self {
        let body = Block::new(node.c_by_n("body"));
        let condition = ParenthesizedExpression::new(node.c_by_n("condition"));
        Self { body, condition }
    }
}

impl<'a> DocBuild<'a> for DoStatement {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt_("do"));
        result.push(self.body.build(b));
        result.push(b._txt_("while"));
        result.push(self.condition.build(b));
        result.push(b.txt(";"));
    }
}

#[derive(Debug, Serialize)]
pub struct WhileStatement {
    pub condition: ParenthesizedExpression,
    pub body: Statement,
}

impl WhileStatement {
    pub fn new(node: Node) -> Self {
        let condition = ParenthesizedExpression::new(node.c_by_n("condition"));
        let body = Statement::new(node.c_by_n("body"));
        Self { condition, body }
    }
}

impl<'a> DocBuild<'a> for WhileStatement {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt_("while"));
        result.push(self.condition.build(b));

        match self.body {
            Statement::SemiColumn => result.push(b.txt(";")),
            _ => {
                result.push(b.txt(" "));
                result.push(self.body.build(b));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UnaryExpression {
    pub operator: String,
    pub operand: Box<Expression>,
}

impl UnaryExpression {
    pub fn new(node: Node) -> Self {
        let operator = node.cvalue_by_n("operator", source_code());
        let operand = Box::new(Expression::new(node.c_by_n("operand")));
        Self { operator, operand }
    }
}

impl<'a> DocBuild<'a> for UnaryExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt(&self.operator));
        result.push(self.operand.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct FieldAccess {
    pub object: MethodObject,
    pub property_navigation: PropertyNavigation,
    pub field: String,
}

impl FieldAccess {
    pub fn new(node: Node) -> Self {
        let obj_node = node.c_by_n("object");
        let object = if obj_node.kind() == "super" {
            MethodObject::Super(Super {})
        } else {
            MethodObject::Primary(Box::new(PrimaryExpression::new(obj_node)))
        };

        let property_navigation = if node.try_c_by_k("safe_navigaion_operator").is_some() {
            PropertyNavigation::SafeNavigationOperator
        } else {
            PropertyNavigation::Dot
        };

        let field = node.cvalue_by_n("field", source_code());

        Self {
            object,
            property_navigation,
            field,
        }
    }
}

impl<'a> DocBuild<'a> for FieldAccess {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(self.object.build(b));
        result.push(self.property_navigation.build(b));
        result.push(b.txt(&self.field));
    }
}

#[derive(Debug, Serialize)]
pub struct EnumDeclaration {
    pub modifiers: Option<Modifiers>,
    pub name: String,
    pub interface: Option<Interface>,
    pub body: EnumBody,
}

impl EnumDeclaration {
    pub fn new(node: Node) -> Self {
        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let name = node.cvalue_by_n("name", source_code());
        let interface = node.try_c_by_k("interfaces").map(|n| Interface::new(n));
        let body = EnumBody::new(node.c_by_n("body"));
        Self {
            modifiers,
            name,
            interface,
            body,
        }
    }
}

impl<'a> DocBuild<'a> for EnumDeclaration {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.modifiers {
            result.push(n.build(b));
        }
        result.push(b.txt_("enum"));
        result.push(b.txt_(&self.name));

        if let Some(ref n) = self.interface {
            result.push(n.build(b));
        }
        result.push(self.body.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct EnumBody {
    enum_constants: Vec<EnumConstant>,
}

impl EnumBody {
    pub fn new(node: Node) -> Self {
        let enum_constants = node
            .try_cs_by_k("enum_constant")
            .into_iter()
            .map(|n| EnumConstant::new(n))
            .collect();

        Self { enum_constants }
    }
}

impl<'a> DocBuild<'a> for EnumBody {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let docs = b.to_docs(&self.enum_constants);

        if docs.is_empty() {
            return result.push(b.concat(vec![b.txt("{"), b.nl(), b.txt("}")]));
        }

        let sep = Insertable::new(None, Some(","), Some(b.nl()));
        let open = Insertable::new(None, Some("{"), Some(b.nl()));
        let close = Insertable::new(Some(b.nl()), Some("}"), None);
        let doc = b.group(b.surround(&docs, sep, open, close));
        result.push(doc);
    }
}

#[derive(Debug, Serialize)]
pub struct EnumConstant {
    pub modifiers: Option<Modifiers>,
    pub name: String,
}

impl EnumConstant {
    pub fn new(node: Node) -> Self {
        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let name = node.cvalue_by_n("name", source_code());
        Self { modifiers, name }
    }
}

impl<'a> DocBuild<'a> for EnumConstant {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.modifiers {
            result.push(n.build(b));
        }
        result.push(b.txt(&self.name));
    }
}

#[derive(Debug, Serialize)]
pub enum DmlExpression {
    Basic {
        dml_type: DmlType,
        security_mode: Option<DmlSecurityMode>,
        exp: Expression,
    },
    Upsert {
        dml_type: DmlType,
        security_mode: Option<DmlSecurityMode>,
        exp: Expression,
        unannotated: Option<Box<UnannotatedType>>,
    },
    Merge {
        dml_type: DmlType,
        security_mode: Option<DmlSecurityMode>,
        exp: Expression,
        exp_extra: Expression,
    },
}

// TODO: update AST to add placeholder field_names
impl DmlExpression {
    pub fn new(node: Node) -> Self {
        let security_mode = node
            .try_c_by_k("dml_security_mode")
            .map(|n| DmlSecurityMode::new(n));

        let (exp_node, second_node) = DmlExpression::get_two_extra_nodes(node)
            .expect("Can't find expected child node in DmlExpression");

        let dml_type = DmlType::from(node.c_by_k("dml_type").first_c().kind());
        match dml_type {
            DmlType::Merge => {
                return Self::Merge {
                    dml_type,
                    security_mode,
                    exp: Expression::new(exp_node),
                    exp_extra: Expression::new(
                        second_node.expect("Second node in DmlExpression::Merge is missing"),
                    ),
                };
            }
            DmlType::Upsert => {
                let unannotated = second_node.map(|n| Box::new(UnannotatedType::new(n)));
                return Self::Upsert {
                    dml_type,
                    security_mode,
                    exp: Expression::new(exp_node),
                    unannotated,
                };
            }
            _ => {
                return Self::Basic {
                    dml_type,
                    security_mode,
                    exp: Expression::new(exp_node),
                };
            }
        }
    }

    fn get_two_extra_nodes(node: Node) -> Option<(Node, Option<Node>)> {
        let excluded_types: HashSet<&str> = [
            "line_comment",
            "block_comment",
            "dml_security_mode",
            "dml_type",
        ]
        .iter()
        .cloned()
        .collect();

        let mut children_iter = node.children_vec().into_iter();
        let mut first: Option<Node> = None;
        let mut second: Option<Node> = None;

        while let Some(child) = children_iter.next() {
            let child_type = child.kind();

            if excluded_types.contains(child_type) {
                continue;
            }

            if first.is_none() {
                first = Some(child);
            } else if second.is_none() {
                second = Some(child);
                break;
            }
        }
        first.map(|f| (f, second))
    }
}

impl<'a> DocBuild<'a> for DmlExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Basic {
                dml_type,
                security_mode,
                exp,
            } => {
                result.push(b.txt_(dml_type.as_str()));
                if let Some(ref s) = security_mode {
                    result.push(s.build(b));
                    result.push(b.txt(" "));
                }
                result.push(exp.build(b));
            }
            Self::Merge {
                dml_type,
                security_mode,
                exp,
                exp_extra,
            } => {
                result.push(b.txt_(dml_type.as_str()));
                if let Some(ref s) = security_mode {
                    result.push(s.build(b));
                    result.push(b.txt(" "));
                }

                let docs = b.to_docs(vec![exp, exp_extra]);
                let sep = Insertable::new::<&str>(None, None, Some(b.softline()));
                let doc = b.group_then_indent(b.intersperse(&docs, sep));
                result.push(doc);
            }
            Self::Upsert {
                dml_type,
                security_mode,
                exp,
                unannotated,
            } => {
                result.push(b.txt_(dml_type.as_str()));

                let mut docs = vec![];
                if let Some(ref s) = security_mode {
                    docs.push(s.build(b));
                }

                docs.push(b.dedent(exp.build(b)));

                if let Some(ref u) = unannotated {
                    docs.push(u.build(b));
                }

                let sep = Insertable::new::<&str>(None, None, Some(b.softline()));
                let doc = b.group_then_indent(b.intersperse(&docs, sep));
                result.push(doc);
            }
        }
        result.push(b.nil());
    }
}

#[derive(Debug, Serialize)]
pub enum DmlSecurityMode {
    User(String),
    System(String),
}

impl DmlSecurityMode {
    pub fn new(n: Node) -> Self {
        let child = n.first_c();
        match child.kind() {
            "user" => Self::User(child.value(source_code())),
            "system" => Self::System(child.value(source_code())),
            _ => panic!("## unknown node: {} in DmlSecurityMode ", n.kind().red()),
        }
    }
}

impl<'a> DocBuild<'a> for DmlSecurityMode {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt_("as"));
        match self {
            Self::User(v) => result.push(b.txt(v)),
            Self::System(v) => result.push(b.txt(v)),
        }
    }
}

#[derive(Debug, Serialize)]
pub enum DmlType {
    Insert,
    Update,
    Delete,
    Undelete,
    Merge,
    Upsert,
}

impl From<&str> for DmlType {
    fn from(t: &str) -> Self {
        match t {
            "insert" => DmlType::Insert,
            "update" => DmlType::Update,
            "delete" => DmlType::Delete,
            "undelete" => DmlType::Undelete,
            "merge" => DmlType::Merge,
            "upsert" => DmlType::Upsert,
            _ => panic!("## unknown node: {} in DmlExpression dml_type ", t.red()),
        }
    }
}

impl DmlType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DmlType::Insert => "insert",
            DmlType::Update => "update",
            DmlType::Delete => "delete",
            DmlType::Undelete => "undelete",
            DmlType::Merge => "merge",
            DmlType::Upsert => "upsert",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ArrayAccess {
    pub array: PrimaryExpression,
    pub index: Expression,
}

impl ArrayAccess {
    pub fn new(node: Node) -> Self {
        let array = PrimaryExpression::new(node.c_by_n("array"));
        let index = Expression::new(node.c_by_n("index"));
        Self { array, index }
    }
}

impl<'a> DocBuild<'a> for ArrayAccess {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(self.array.build(b));
        result.push(b.txt("["));
        result.push(self.index.build(b));
        result.push(b.txt("]"));
    }
}

#[derive(Debug, Serialize)]
pub struct ArrayCreationExpression {
    pub type_: SimpleType,
    pub variant: ArrayCreationVariant,
}

impl ArrayCreationExpression {
    pub fn new(node: Node) -> Self {
        assert_check(node, "array_creation_expression");

        let type_ = SimpleType::new(node.c_by_n("type"));

        let value_node = node.try_c_by_n("value");
        let dimensions_node = node.try_c_by_n("dimensions");

        let variant = if value_node.is_none() {
            // DD
            let dimensions_exprs = node
                .cs_by_k("dimensions_expr")
                .into_iter()
                .map(|n| DimensionsExpr::new(n))
                .collect();
            let dimensions = node.try_c_by_k("dimensions").map(|n| Dimensions::new(n));
            ArrayCreationVariant::DD {
                dimensions_exprs,
                dimensions,
            }
        } else if dimensions_node.is_none() {
            //OnlyV
            let value = ArrayInitializer::new(node.c_by_n("value"));
            ArrayCreationVariant::OnlyV { value }
        } else {
            //DV
            ArrayCreationVariant::DV {
                value: ArrayInitializer::new(value_node.unwrap()),
                dimensions: Dimensions::new(dimensions_node.unwrap()),
            }
        };

        Self { type_, variant }
    }
}

impl<'a> DocBuild<'a> for ArrayCreationExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt_("new"));
        result.push(self.type_.build(b));
        result.push(self.variant.build(b));
    }
}

#[derive(Debug, Serialize)]
pub enum ArrayCreationVariant {
    DD {
        dimensions_exprs: Vec<DimensionsExpr>,
        dimensions: Option<Dimensions>,
    },
    DV {
        dimensions: Dimensions,
        value: ArrayInitializer,
    },
    OnlyV {
        value: ArrayInitializer,
    },
}

impl<'a> DocBuild<'a> for ArrayCreationVariant {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::OnlyV { value } => {
                result.push(value.build(b));
            }
            Self::DD {
                dimensions_exprs,
                dimensions,
            } => {
                dimensions_exprs
                    .iter()
                    .for_each(|n| result.push(n.build(b)));

                if let Some(ref n) = dimensions {
                    result.push(b.txt(" "));
                    result.push(n.build(b));
                }
            }
            Self::DV { dimensions, value } => {
                result.push(dimensions.build(b));
                result.push(value.build(b));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Dimensions {}

impl Dimensions {
    pub fn new(node: Node) -> Self {
        assert_check(node, "dimensions");
        Self {}
    }
}

impl<'a> DocBuild<'a> for Dimensions {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("[]"));
    }
}

#[derive(Debug, Serialize)]
pub struct DimensionsExpr {
    pub exp: Expression,
}

impl DimensionsExpr {
    pub fn new(node: Node) -> Self {
        assert_check(node, "dimensions_expr");

        let exp = Expression::new(node.first_c());
        Self { exp }
    }
}

impl<'a> DocBuild<'a> for DimensionsExpr {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("["));
        result.push(self.exp.build(b));
        result.push(b.txt("]"));
    }
}

#[derive(Debug, Serialize)]
pub struct ReturnStatement {
    pub exp: Option<Expression>,
}

impl ReturnStatement {
    pub fn new(node: Node) -> Self {
        let exp = node.try_first_c().map(|n| Expression::new(n));
        Self { exp }
    }
}

impl<'a> DocBuild<'a> for ReturnStatement {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("return"));
        if let Some(ref exp) = self.exp {
            result.push(b.txt(" "));
            result.push(exp.build(b));
        }
        result.push(b.txt(";"));
    }
}

#[derive(Debug, Serialize)]
pub struct TernaryExpression {
    pub condition: Expression,
    pub consequence: Expression,
    pub alternative: Expression,
}

impl TernaryExpression {
    pub fn new(node: Node) -> Self {
        let condition = Expression::new(node.c_by_n("condition"));
        let consequence = Expression::new(node.c_by_n("consequence"));
        let alternative = Expression::new(node.c_by_n("alternative"));
        Self {
            condition,
            consequence,
            alternative,
        }
    }
}

impl<'a> DocBuild<'a> for TernaryExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(self.condition.build(b));
        result.push(b._txt_("?"));
        result.push(self.consequence.build(b));
        result.push(b._txt_(":"));
        result.push(self.alternative.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct TryStatement {
    pub body: Block,
    pub tail: TryStatementTail,
}

impl TryStatement {
    pub fn new(node: Node) -> Self {
        assert_check(node, "try_statement");

        let body = Block::new(node.c_by_n("body"));
        let tail = if node.try_c_by_k("finally_clause").is_some() {
            TryStatementTail::CatchesFinally(
                node.try_cs_by_k("catch_clause")
                    .into_iter()
                    .map(|n| CatchClause::new(n))
                    .collect(),
                FinallyClause::new(node.c_by_k("finally_clause")),
            )
        } else {
            TryStatementTail::Catches(
                node.cs_by_k("catch_clause")
                    .into_iter()
                    .map(|n| CatchClause::new(n))
                    .collect(),
            )
        };
        Self { body, tail }
    }
}

impl<'a> DocBuild<'a> for TryStatement {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt_("try"));
        result.push(self.body.build(b));
        result.push(self.tail.build(b));
    }
}

#[derive(Debug, Serialize)]
pub enum TryStatementTail {
    Catches(Vec<CatchClause>),
    CatchesFinally(Vec<CatchClause>, FinallyClause),
}

impl<'a> DocBuild<'a> for TryStatementTail {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Catches(v) => {
                let docs = b.to_docs(v);
                let catches_doc = b.concat(docs);
                result.push(catches_doc);
            }
            Self::CatchesFinally(v, f) => {
                let docs = b.to_docs(v);
                let catches_doc = b.concat(docs);
                result.push(catches_doc);
                result.push(f.build(b));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CatchClause {
    pub formal_parameter: CatchFormalParameter,
    pub body: Block,
}

impl CatchClause {
    pub fn new(node: Node) -> Self {
        assert_check(node, "catch_clause");

        let formal_parameter = CatchFormalParameter::new(node.c_by_k("catch_formal_parameter"));
        let body = Block::new(node.c_by_n("body"));
        Self {
            formal_parameter,
            body,
        }
    }
}

impl<'a> DocBuild<'a> for CatchClause {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b._txt_("catch"));

        result.push(b.txt("("));
        result.push(self.formal_parameter.build(b));
        result.push(b.txt_(")"));
        result.push(self.body.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct CatchFormalParameter {
    pub modifiers: Option<Modifiers>,
    pub type_: UnannotatedType,
    pub name: String,
    pub dimensions: Option<Dimensions>,
}

impl CatchFormalParameter {
    pub fn new(node: Node) -> Self {
        assert_check(node, "catch_formal_parameter");

        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));

        // TODO: can't locate "UnannotatedType" which is an internal type;
        let type_node = node
            .c_by_n("name")
            .prev_named_sibling()
            .expect("missing mandatory type node in CatchFormalParameter");
        let type_ = UnannotatedType::new(type_node);

        let name = node.cvalue_by_n("name", source_code());
        let dimensions = node.try_c_by_k("dimensions").map(|n| Dimensions::new(n));

        Self {
            modifiers,
            type_,
            name,
            dimensions,
        }
    }
}

impl<'a> DocBuild<'a> for CatchFormalParameter {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.modifiers {
            result.push(n.build(b));
        }
        result.push(self.type_.build(b));
        result.push(b._txt(&self.name));
        if let Some(ref d) = self.dimensions {
            result.push(b.txt(" "));
            result.push(d.build(b));
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FinallyClause {
    pub body: Block,
}

impl FinallyClause {
    pub fn new(node: Node) -> Self {
        assert_check(node, "finally_clause");

        let body = Block::new(node.c_by_k("block"));
        Self { body }
    }
}

impl<'a> DocBuild<'a> for FinallyClause {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b._txt_("finally"));
        result.push(self.body.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct StaticInitializer {
    pub block: Block,
}

impl StaticInitializer {
    pub fn new(node: Node) -> Self {
        Self {
            block: Block::new(node.c_by_k("block")),
        }
    }
}

impl<'a> DocBuild<'a> for StaticInitializer {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt_("static"));
        result.push(self.block.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct InterfaceDeclaration {
    pub modifiers: Option<Modifiers>,
    pub name: String,
    pub type_parameters: Option<TypeParameters>,
    pub extends: Option<ExtendsInterface>,
    pub body: InterfaceBody,
}

impl InterfaceDeclaration {
    pub fn new(node: Node) -> Self {
        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let name = node.cvalue_by_n("name", source_code());
        let type_parameters = node
            .try_c_by_k("type_parameters")
            .map(|n| TypeParameters::new(n));
        let extends = node
            .try_c_by_k("extends_interfaces")
            .map(|n| ExtendsInterface::new(n));
        let body = InterfaceBody::new(node.c_by_n("body"));

        Self {
            modifiers,
            name,
            type_parameters,
            extends,
            body,
        }
    }
}

impl<'a> DocBuild<'a> for InterfaceDeclaration {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.modifiers {
            result.push(n.build(b));
        }

        result.push(b.txt_("interface"));
        result.push(b.txt(&self.name));

        if let Some(ref n) = self.type_parameters {
            result.push(n.build(b));
        }

        if let Some(ref n) = self.extends {
            result.push(n.build(b));
        }

        result.push(b.txt(" "));
        result.push(self.body.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct ExtendsInterface {
    pub types: Vec<Type>,
}

impl ExtendsInterface {
    pub fn new(node: Node) -> Self {
        let types = node
            .c_by_k("type_list")
            .children_vec()
            .into_iter()
            .map(|n| Type::new(n))
            .collect();
        Self { types }
    }
}

impl<'a> DocBuild<'a> for ExtendsInterface {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let docs = b.to_docs(&self.types);
        let sep = Insertable::new(None, Some(", "), None);
        let doc = b.intersperse(&docs, sep);

        let extends_group = b.concat(vec![b._txt_("extends"), doc]);
        result.push(extends_group);
    }
}

#[derive(Debug, Serialize)]
pub struct InterfaceBody {
    members: Vec<BodyMember<InterfaceMember>>,
}

impl InterfaceBody {
    pub fn new(node: Node) -> Self {
        assert_check(node, "interface_body");

        let members: Vec<_> = node
            .children_vec()
            .into_iter()
            .map(|n| {
                let member = match n.kind() {
                    "constant_declaration" => {
                        InterfaceMember::Constant(ConstantDeclaration::new(n))
                    }
                    "enum_declaration" => InterfaceMember::EnumD(EnumDeclaration::new(n)),
                    "method_declaration" => InterfaceMember::Method(MethodDeclaration::new(n)),
                    "class_declaration" => InterfaceMember::Class(ClassDeclaration::new(n)),
                    "interface_declaration" => {
                        InterfaceMember::Interface(InterfaceDeclaration::new(n))
                    }
                    _ => panic!("## unknown node: {} in InterfaceBody", n.kind().red()),
                };

                BodyMember {
                    member,
                    has_trailing_newline: has_trailing_new_line(&n),
                }
            })
            .collect();

        Self { members }
    }
}

impl<'a> DocBuild<'a> for InterfaceBody {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.surround_body(&self.members, "{", "}"));
    }
}

#[derive(Debug, Serialize)]
pub enum InterfaceMember {
    Constant(ConstantDeclaration),
    EnumD(EnumDeclaration),
    Method(MethodDeclaration),
    Class(ClassDeclaration),
    Interface(InterfaceDeclaration),
    Semicolon,
}

impl<'a> DocBuild<'a> for InterfaceMember {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::Constant(n) => {
                result.push(n.build(b));
            }
            Self::EnumD(n) => {
                result.push(n.build(b));
            }
            Self::Method(n) => {
                result.push(n.build(b));
            }
            Self::Class(n) => {
                result.push(n.build(b));
            }
            Self::Interface(n) => {
                result.push(n.build(b));
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConstantDeclaration {
    pub modifiers: Option<Modifiers>,
    pub type_: UnannotatedType,
    pub declarators: Vec<VariableDeclarator>,
}

impl ConstantDeclaration {
    pub fn new(node: Node) -> Self {
        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let type_ = UnannotatedType::new(node.c_by_n("type"));
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

impl<'a> DocBuild<'a> for ConstantDeclaration {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.modifiers {
            result.push(n.build(b));
        }

        result.push(self.type_.build(b));
        result.push(b.txt(" "));

        let docs = b.to_docs(&self.declarators);
        let sep = Insertable::new(None, Some(","), Some(b.softline()));
        let doc = b.group_then_indent(b.intersperse(&docs, sep));
        result.push(doc);
        result.push(b.txt(";"));
    }
}

#[derive(Debug, Serialize)]
pub struct AccessorList {
    pub accessor_declarations: Vec<AccessorDeclaration>,
    pub child_has_body_section: bool,
}

impl AccessorList {
    pub fn new(node: Node) -> Self {
        let accessor_declarations: Vec<_> = node
            .cs_by_k("accessor_declaration")
            .into_iter()
            .map(|n| AccessorDeclaration::new(n))
            .collect();

        let child_has_body_section = accessor_declarations.iter().any(|n| n.body.is_some());

        Self {
            accessor_declarations,
            child_has_body_section,
        }
    }
}

impl<'a> DocBuild<'a> for AccessorList {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let docs = b.to_docs(&self.accessor_declarations);

        // to align with prettier apex;
        if self.child_has_body_section {
            let sep = Insertable::new::<&str>(None, None, Some(b.nl()));
            let open = Insertable::new(None, Some("{"), Some(b.nl()));
            let close = Insertable::new(Some(b.nl()), Some("}"), None);
            let doc = b.group(b.surround(&docs, sep, open, close));
            result.push(doc);
        } else {
            let sep = Insertable::new::<&str>(None, None, Some(b.softline()));
            let open = Insertable::new(None, Some("{"), Some(b.softline()));
            let close = Insertable::new(Some(b.softline()), Some("}"), None);
            let doc = b.group(b.surround(&docs, sep, open, close));
            result.push(doc);
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AccessorDeclaration {
    pub modifiers: Option<Modifiers>,
    pub accessor: String,
    pub body: Option<Block>,
}

impl AccessorDeclaration {
    pub fn new(node: Node) -> Self {
        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));
        let accessor = node.cvalue_by_n("accessor", source_code());
        let body = node.try_c_by_n("body").map(|n| Block::new(n));
        Self {
            modifiers,
            accessor,
            body,
        }
    }
}

impl<'a> DocBuild<'a> for AccessorDeclaration {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.modifiers {
            result.push(n.build(b));
        }
        result.push(b.txt(&self.accessor));

        if let Some(ref n) = self.body {
            result.push(b.txt(" "));
            result.push(n.build(b));
        } else {
            result.push(b.txt(";"));
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CastExpression {
    pub type_: Type,
    pub value: Expression,
}

impl CastExpression {
    pub fn new(node: Node) -> Self {
        let type_ = Type::new(node.c_by_n("type"));
        let value = Expression::new(node.c_by_n("value"));
        Self { type_, value }
    }
}

impl<'a> DocBuild<'a> for CastExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("("));
        result.push(self.type_.build(b));
        result.push(b.txt_(")"));
        result.push(self.value.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct ThrowStatement {
    pub exp: Expression,
}

impl ThrowStatement {
    pub fn new(node: Node) -> Self {
        let exp = Expression::new(node.first_c());
        Self { exp }
    }
}

impl<'a> DocBuild<'a> for ThrowStatement {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("throw "));
        result.push(self.exp.build(b));
        result.push(b.txt(";"));
    }
}

#[derive(Debug, Serialize)]
pub struct BreakStatement {
    pub identifier: Option<String>,
}

impl BreakStatement {
    pub fn new(node: Node) -> Self {
        let identifier = node
            .try_c_by_k("identifier")
            .map(|n| n.value(source_code()));
        Self { identifier }
    }
}

impl<'a> DocBuild<'a> for BreakStatement {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("break"));

        if let Some(ref n) = self.identifier {
            result.push(b.txt(" "));
            result.push(b.txt(&n));
        }
        result.push(b.txt(";"));
    }
}

#[derive(Debug, Serialize)]
pub struct ContinueStatement {
    pub identifier: Option<String>,
}

impl ContinueStatement {
    pub fn new(node: Node) -> Self {
        let identifier = node
            .try_c_by_k("identifier")
            .map(|n| n.value(source_code()));
        Self { identifier }
    }
}

impl<'a> DocBuild<'a> for ContinueStatement {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("continue"));

        if let Some(ref n) = self.identifier {
            result.push(b.txt(" "));
            result.push(b.txt(&n));
        }
        result.push(b.txt(";"));
    }
}

#[derive(Debug, Serialize)]
pub struct SwitchExpression {
    pub condition: Expression,
    pub body: SwitchBlock,
}

impl SwitchExpression {
    pub fn new(node: Node) -> Self {
        let condition = Expression::new(node.c_by_n("condition"));
        let body = SwitchBlock::new(node.c_by_n("body"));
        Self { condition, body }
    }
}

impl<'a> DocBuild<'a> for SwitchExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt_("switch on"));
        result.push(self.condition.build(b));
        result.push(b.txt(" "));
        result.push(self.body.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct SwitchBlock {
    pub rules: Vec<SwitchRule>,
}

impl SwitchBlock {
    pub fn new(node: Node) -> Self {
        let rules = node
            .cs_by_k("switch_rule")
            .into_iter()
            .map(|n| SwitchRule::new(n))
            .collect();
        Self { rules }
    }
}

impl<'a> DocBuild<'a> for SwitchBlock {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let docs = b.to_docs(&self.rules);

        let sep = Insertable::new(None, Some(""), Some(b.nl()));
        let open = Insertable::new(None, Some("{"), Some(b.nl()));
        let close = Insertable::new(Some(b.nl()), Some("}"), None);
        let doc = b.surround(&docs, sep, open, close);
        result.push(doc);
    }
}

#[derive(Debug, Serialize)]
pub struct SwitchRule {
    pub label: SwitchLabel,
    pub block: Block,
}

impl SwitchRule {
    // TODO: update parser
    pub fn new(node: Node) -> Self {
        let label_node = node.c_by_k("switch_label");

        let label = if label_node.children_vec().len() == 0 {
            SwitchLabel::Else
        } else if label_node.try_c_by_k("identifier").is_some() {
            let mut sobjects = Vec::new();
            let mut current_type: Option<UnannotatedType> = None;

            for child in label_node.children_vec() {
                match child.kind() {
                    "identifier" => {
                        sobjects.push(SObjectVar {
                            unannotated_type: current_type.take(),
                            identifier: child.value(source_code()),
                        });
                    }
                    _ => {
                        current_type = Some(UnannotatedType::new(child));
                    }
                }
            }

            SwitchLabel::SObjects(sobjects)
        } else {
            let expressions = label_node
                .children_vec()
                .into_iter()
                .map(|n| Expression::new(n))
                .collect();

            SwitchLabel::Expressions(expressions)
        };
        let block = Block::new(node.c_by_k("block"));
        Self { label, block }
    }
}

impl<'a> DocBuild<'a> for SwitchRule {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(self.label.build(b));
        result.push(b.txt(" "));
        result.push(self.block.build(b));
    }
}

#[derive(Debug, Serialize)]
pub enum SwitchLabel {
    SObjects(Vec<SObjectVar>),
    Expressions(Vec<Expression>),
    Else,
}

impl<'a> DocBuild<'a> for SwitchLabel {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt_("when"));
        match self {
            Self::SObjects(vec) => {
                let docs = b.to_docs(vec);
                let sep = Insertable::new(None, Some(", "), None);
                let doc = b.intersperse(&docs, sep);
                result.push(doc);
            }
            Self::Expressions(vec) => {
                let docs = b.to_docs(vec);
                let sep = Insertable::new(None, Some(", "), None);
                let doc = b.intersperse(&docs, sep);
                result.push(doc);
            }
            Self::Else => {
                result.push(b.txt("else"));
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SObjectVar {
    pub unannotated_type: Option<UnannotatedType>,
    pub identifier: String,
}

impl<'a> DocBuild<'a> for SObjectVar {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.unannotated_type {
            result.push(n.build(b));
            result.push(b.txt(" "));
        }
        result.push(b.txt(&self.identifier));
    }
}

#[derive(Debug, Serialize)]
pub struct InstanceOfExpression {
    pub left: Expression,
    pub right: Type,
}

impl InstanceOfExpression {
    pub fn new(node: Node) -> Self {
        let left = Expression::new(node.c_by_n("left"));
        let right = Type::new(node.c_by_n("right"));
        Self { left, right }
    }
}

impl<'a> DocBuild<'a> for InstanceOfExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(self.left.build(b));
        result.push(b._txt_("instanceof"));
        result.push(self.right.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct VersionExpression {}

impl VersionExpression {
    pub fn new(_node: Node) -> Self {
        Self {}
    }
}

impl<'a> DocBuild<'a> for VersionExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        // TODO: when it's major.minor? No corresponding nodes from parser
        result.push(b.txt("Package.Version.Request"));
    }
}

#[derive(Debug, Serialize)]
pub struct JavaFieldAccess {
    pub field_access: FieldAccess,
}

impl JavaFieldAccess {
    pub fn new(node: Node) -> Self {
        let field_access = FieldAccess::new(node.c_by_k("field_access"));
        Self { field_access }
    }
}

impl<'a> DocBuild<'a> for JavaFieldAccess {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("java:"));
        result.push(self.field_access.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct JavaType {
    pub scoped_type_identifier: ScopedTypeIdentifier,
}

impl JavaType {
    pub fn new(node: Node) -> Self {
        let scoped_type_identifier =
            ScopedTypeIdentifier::new(node.c_by_k("scoped_type_identifier"));
        Self {
            scoped_type_identifier,
        }
    }
}

impl<'a> DocBuild<'a> for JavaType {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("java:"));
        result.push(self.scoped_type_identifier.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct ArrayType {
    pub element: UnannotatedType,
    pub dimensions: Dimensions,
}

impl ArrayType {
    pub fn new(node: Node) -> Self {
        assert_check(node, "array_type");

        let element = UnannotatedType::new(node.c_by_n("element"));
        let dimensions = Dimensions::new(node.c_by_n("dimensions"));
        Self {
            element,
            dimensions,
        }
    }
}

impl<'a> DocBuild<'a> for ArrayType {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(self.element.build(b));
        result.push(self.dimensions.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct TriggerDeclaration {
    pub name: String,
    pub object: String,
    pub events: Vec<TriggerEvent>,
    pub body: TriggerBody,
}

impl TriggerDeclaration {
    pub fn new(node: Node) -> Self {
        let name = node.cvalue_by_n("name", source_code());
        let object = node.cvalue_by_n("object", source_code());
        let events = node
            .cs_by_n("events")
            .into_iter()
            .map(|n| TriggerEvent::new(n.first_c()))
            .collect();
        let body = TriggerBody::new(node.c_by_n("body"));
        Self {
            name,
            object,
            events,
            body,
        }
    }
}

impl<'a> DocBuild<'a> for TriggerDeclaration {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt_("trigger"));
        result.push(b.txt(&self.name));
        result.push(b._txt_("on"));
        result.push(b.txt(&self.object));

        let docs = b.to_docs(&self.events);

        let sep = Insertable::new(None, Some(","), Some(b.softline()));
        let open = Insertable::new(None, Some("("), Some(b.maybeline()));
        let close = Insertable::new(Some(b.maybeline()), Some(")"), None);
        let doc = b.group(b.surround(&docs, sep, open, close));
        result.push(doc);

        result.push(b.txt(" "));
        result.push(self.body.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct TriggerBody {
    pub block: Block,
}

impl TriggerBody {
    pub fn new(node: Node) -> Self {
        let block = Block::new(node.c_by_k("block"));
        Self { block }
    }
}

impl<'a> DocBuild<'a> for TriggerBody {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(self.block.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct QueryExpression {
    pub query_body: QueryBody,
}

impl QueryExpression {
    pub fn new(node: Node) -> Self {
        let query_body = QueryBody::SOQL(SoqlQueryBody::new(node.first_c()));
        Self { query_body }
    }
}

impl<'a> DocBuild<'a> for QueryExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let docs_to_indent = vec![b.txt("["), b.maybeline(), self.query_body.build(b)];
        let first_part = b.indent(b.concat(docs_to_indent));

        let doc = b.group(b.concat(vec![first_part, b.maybeline(), b.txt("]")]));
        result.push(doc);
    }
}

#[derive(Debug, Serialize)]
pub enum QueryBody {
    SOQL(SoqlQueryBody),
    SOSL,
}

impl QueryBody {
    pub fn new(node: Node) -> Self {
        match node.kind() {
            "soql_query_body" => Self::SOQL(SoqlQueryBody::new(node)),
            "sosl_query_body" => unimplemented!(),
            _ => unimplemented!(),
        }
    }
}

impl<'a> DocBuild<'a> for QueryBody {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        match self {
            Self::SOQL(n) => {
                result.push(n.build(b));
            }
            Self::SOSL => {
                unimplemented!()
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SoqlQueryBody {
    pub select_clause: SelectClause,
    pub from_clause: FromClause,
    //using_clause;
    pub where_clause: Option<WhereClause>,
    //with_c;
    //group_by_c;
    pub order_by_clause: Option<OrderByClause>,
    pub limit_clause: Option<LimitClause>,
    pub offset_clause: Option<OffsetClause>,
    pub for_clause: Vec<String>,
    //update_c;
    pub all_rows_clause: Option<()>,
}

impl SoqlQueryBody {
    pub fn new(node: Node) -> Self {
        let select_clause = SelectClause::new(node.c_by_n("select_clause"));
        let from_clause = FromClause::new(node.c_by_n("from_clause"));
        let where_clause = node.try_c_by_n("where_clause").map(|n| WhereClause::new(n));
        let order_by_clause = node
            .try_c_by_n("order_by_clause")
            .map(|n| OrderByClause::new(n));
        let limit_clause = node.try_c_by_n("limit_clause").map(|n| LimitClause::new(n));
        let offset_clause = node
            .try_c_by_n("offset_clause")
            .map(|n| OffsetClause::new(n));
        let all_rows_clause = node.try_c_by_n("all_rows_clause").map(|_| ());
        let for_clause = node
            .try_cs_by_k("for_clause")
            .into_iter()
            .map(|n| n.cvalue_by_k("for_type", source_code()))
            .collect();

        Self {
            select_clause,
            from_clause,
            where_clause,
            order_by_clause,
            limit_clause,
            offset_clause,
            for_clause,
            all_rows_clause,
        }
    }
}

impl<'a> DocBuild<'a> for SoqlQueryBody {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let mut docs = vec![];
        docs.push(self.select_clause.build(b));
        docs.push(self.from_clause.build(b));

        if let Some(ref n) = self.where_clause {
            docs.push(n.build(b));
        }
        if let Some(ref n) = self.order_by_clause {
            docs.push(n.build(b));
        }
        if let Some(ref n) = self.limit_clause {
            docs.push(n.build(b));
        }
        if let Some(ref n) = self.offset_clause {
            docs.push(n.build(b));
        }
        if let Some(_) = self.all_rows_clause {
            docs.push(b.txt("ALL ROWS"));
        }
        if !self.for_clause.is_empty() {
            let for_types: Vec<DocRef<'_>> = self.for_clause.iter().map(|n| b.txt(n)).collect();
            let sep = Insertable::new(None, Some(", "), None);
            let for_types_doc = b.intersperse(&for_types, sep);

            let for_clause_doc = b.concat(vec![b.txt_("FOR"), for_types_doc]);
            docs.push(for_clause_doc);
        }

        let sep = Insertable::new::<&str>(None, None, Some(b.softline()));
        let doc = b.intersperse(&docs, sep); // to align with prettier apex, no group_then_indent()
        result.push(doc);
    }
}

#[derive(Debug, Serialize)]
pub struct FromClause {
    pub content: StorageVariant,
}

impl FromClause {
    pub fn new(node: Node) -> Self {
        let content = StorageVariant::new(node.first_c());
        Self { content }
    }
}

impl<'a> DocBuild<'a> for FromClause {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt_("FROM"));
        result.push(self.content.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct StorageAlias {
    pub storage_alias: StorageIdentifier,
    pub identifier: String,
}

impl StorageAlias {
    pub fn new(node: Node) -> Self {
        assert_check(node, "storage_alias");

        let storage_alias = StorageIdentifier::new(node.c_by_k("storage_alias"));
        let identifier = node.cvalue_by_k("identifier", source_code());
        Self {
            storage_alias,
            identifier,
        }
    }
}

impl<'a> DocBuild<'a> for StorageAlias {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(self.storage_alias.build(b));
        result.push(b._txt_("AS"));
        result.push(b.txt(&self.identifier));
    }
}

#[derive(Debug, Serialize)]
pub struct LimitClause {
    pub limit_value: LimitValue,
}

impl LimitClause {
    pub fn new(node: Node) -> Self {
        let limit_value = LimitValue::new(node.first_c());
        Self { limit_value }
    }
}

impl<'a> DocBuild<'a> for LimitClause {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt_("LIMIT"));
        result.push(self.limit_value.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct BoundApexExpression {
    pub exp: Box<Expression>,
}

impl BoundApexExpression {
    pub fn new(node: Node) -> Self {
        assert_check(node, "bound_apex_expression");
        let exp = Box::new(Expression::new(node.first_c()));
        Self { exp }
    }
}

impl<'a> DocBuild<'a> for BoundApexExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt(":"));
        result.push(self.exp.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct WhereClause {
    pub boolean_exp: BooleanExpression,
}

impl WhereClause {
    pub fn new(node: Node) -> Self {
        assert_check(node, "where_clause");
        let boolean_exp = BooleanExpression::new(node.first_c());
        Self { boolean_exp }
    }
}

impl<'a> DocBuild<'a> for WhereClause {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let mut docs = vec![];
        docs.push(b.txt("WHERE"));
        docs.push(b.softline());
        docs.push(self.boolean_exp.build(b));

        result.push(b.group_then_indent(b.concat(docs)));
    }
}

#[derive(Debug, Serialize)]
pub struct FunctionExpression {
    pub function_variant: FunctionVariant,
}

//impl FunctionExpression {
//    pub fn new(node: Node) -> Self {}
//}

impl<'a> DocBuild<'a> for FunctionExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {}
}

#[derive(Debug, Serialize)]
pub struct ComparisonExpression {
    pub value: Box<ValueExpression>,
    pub comparison: Comparison,
}

impl ComparisonExpression {
    pub fn new(node: Node) -> Self {
        assert_check(node, "comparison_expression");

        let value = Box::new(ValueExpression::new(node.first_c()));
        let comparison = ComparisonExpression::get_comparsion(&node);
        Self { value, comparison }
    }

    fn get_comparsion(node: &Node) -> Comparison {
        if let Some(operator_node) = node.try_c_by_k("value_comparison_operator") {
            let next_node = operator_node.next_named();
            let compared_with = match next_node.kind() {
                "bound_apex_expression" => {
                    ValueComparedWith::Bound(BoundApexExpression::new(next_node))
                }
                _ => ValueComparedWith::Literal(SoqlLiteral::new(next_node)),
            };

            Comparison::Value(ValueComparison {
                operator: operator_node.value(source_code()),
                compared_with,
            })
        } else if let Some(operator_node) = node.try_c_by_k("set_comparison_operator") {
            let next_node = operator_node.next_named();
            Comparison::Set(SetComparison {
                operator: operator_node.value(source_code()),
                set_value: SetValue::new(next_node),
            })
        } else {
            unreachable!()
        }
    }
}

impl<'a> DocBuild<'a> for ComparisonExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(self.value.build(b));
        result.push(self.comparison.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct ValueComparison {
    pub operator: String,
    pub compared_with: ValueComparedWith,
}

impl<'a> DocBuild<'a> for ValueComparison {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b._txt_(&self.operator));
        result.push(self.compared_with.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct SetComparison {
    pub operator: String,
    pub set_value: SetValue,
}

impl<'a> DocBuild<'a> for SetComparison {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b._txt_(&self.operator));
        result.push(self.set_value.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct ComparableList {
    pub values: Vec<ComparableListValue>,
}

impl ComparableList {
    pub fn new(node: Node) -> Self {
        let values = node
            .children_vec()
            .into_iter()
            .map(|n| ComparableListValue::new(n))
            .collect();
        Self { values }
    }
}

impl<'a> DocBuild<'a> for ComparableList {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let docs = b.to_docs(&self.values);

        let sep = Insertable::new(None, Some(", "), None);
        let open = Insertable::new(None, Some("("), None);
        let close = Insertable::new(None, Some(")"), None);
        let doc = b.surround(&docs, sep, open, close);
        result.push(doc);
    }
}

#[derive(Debug, Serialize)]
pub struct OrderByClause {
    pub exps: Vec<OrderExpression>,
}

impl OrderByClause {
    pub fn new(node: Node) -> Self {
        assert_check(node, "order_by_clause");
        let exps = node
            .cs_by_k("order_expression")
            .into_iter()
            .map(|n| OrderExpression::new(n))
            .collect();
        Self { exps }
    }
}

impl<'a> DocBuild<'a> for OrderByClause {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt_("ORDER BY"));

        let docs = b.to_docs(&self.exps);
        let sep = Insertable::new(None, Some(", "), None);
        let doc = b.intersperse(&docs, sep);
        result.push(doc);
    }
}

#[derive(Debug, Serialize)]
pub struct OrderExpression {
    pub value_expression: ValueExpression,
    pub direction: Option<String>,
    pub null_direction: Option<String>,
}

impl OrderExpression {
    pub fn new(node: Node) -> Self {
        assert_check(node, "order_expression");

        let value_expression = ValueExpression::new(node.first_c());
        let direction = node
            .try_c_by_k("order_direction")
            .map(|n| n.value(source_code()));
        let null_direction = node
            .try_c_by_k("order_null_direction")
            .map(|n| n.value(source_code()));

        Self {
            value_expression,
            direction,
            null_direction,
        }
    }
}

impl<'a> DocBuild<'a> for OrderExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(self.value_expression.build(b));

        if let Some(ref n) = self.direction {
            result.push(b._txt(n));
        }
        if let Some(ref n) = self.null_direction {
            result.push(b._txt(n));
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SubQuery {
    pub soql_query_body: Box<SoqlQueryBody>,
}

impl SubQuery {
    pub fn new(node: Node) -> Self {
        let soql_query_body = Box::new(SoqlQueryBody::new(node.c_by_k("soql_query_body")));
        Self { soql_query_body }
    }
}

impl<'a> DocBuild<'a> for SubQuery {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("("));
        result.push(b.flat(self.soql_query_body.build(b)));
        result.push(b.txt(")"));
    }
}

#[derive(Debug, Serialize)]
pub struct MapCreationExpression {
    type_: SimpleType,
    value: MapInitializer,
}

impl MapCreationExpression {
    pub fn new(node: Node) -> Self {
        assert_check(node, "map_creation_expression");

        let type_ = SimpleType::new(node.c_by_n("type"));
        let value = MapInitializer::new(node.c_by_n("value"));
        Self { type_, value }
    }
}

impl<'a> DocBuild<'a> for MapCreationExpression {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt_("new"));
        result.push(self.type_.build(b));
        result.push(self.value.build(b));
    }
}

#[derive(Debug, Serialize)]
pub struct MapInitializer {
    initializers: Vec<MapKeyInitializer>,
}

impl MapInitializer {
    pub fn new(node: Node) -> Self {
        assert_check(node, "map_initializer");

        let initializers: Vec<_> = node
            .children_vec()
            .into_iter()
            .map(|n| MapKeyInitializer::new(n))
            .collect();

        Self { initializers }
    }
}

impl<'a> DocBuild<'a> for MapInitializer {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let docs = b.to_docs(&self.initializers);

        let sep = Insertable::new(None, Some(","), Some(b.softline()));
        let open = Insertable::new(None, Some("{"), Some(b.softline()));
        let close = Insertable::new(Some(b.softline()), Some("}"), None);
        let doc = b.group(b.surround(&docs, sep, open, close));
        result.push(doc);
    }
}

#[derive(Debug, Serialize)]
pub struct MapKeyInitializer {
    pub exp1: Box<Expression>,
    pub exp2: Box<Expression>,
}

impl MapKeyInitializer {
    pub fn new(node: Node) -> Self {
        assert_check(node, "map_key_initializer");

        let children = node.children_vec();
        if children.len() != 2 {
            panic!("### must be exactly 2 child nodes in MapKeyInitializer");
        }
        let exp1 = Box::new(Expression::new(children[0]));
        let exp2 = Box::new(Expression::new(children[1]));
        Self { exp1, exp2 }
    }
}

impl<'a> DocBuild<'a> for MapKeyInitializer {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(self.exp1.build(b));
        result.push(b._txt_("=>"));
        result.push(self.exp2.build(b));
    }
}
