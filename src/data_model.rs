use crate::{
    accessor::Accessor,
    doc::DocRef,
    doc_builder::DocBuilder,
    enum_def::*,
    utility::{assert_check, source_code},
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
        let member_docs = b.build_docs(&self.members);
        let body_doc = b.sep_multi_line(&member_docs, "");
        result.push(body_doc);
        result.push(b.nl());
    }
}

#[derive(Debug, Serialize)]
pub struct ClassDeclaration {
    pub buckets: Option<CommentBuckets>,
    pub modifiers: Option<Modifiers>,
    pub name: String,
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
        let interface = node.try_c_by_k("interfaces").map(|n| Interface::new(n));
        let body = ClassBody::new(node.c_by_n("body"));
        let range = DataRange::from(node.range());

        Self {
            buckets,
            modifiers,
            name,
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

        result.push(b.txt("class "));
        result.push(b.txt(&self.name));

        if let Some(ref n) = self.interface {
            result.push(b.txt(" "));
            result.push(n.build(b));
        }

        result.push(b.txt(" {"));

        if self.body.declarations.is_empty() {
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

#[derive(Debug, Default, Serialize)]
pub struct Modifiers {
    //pub buckets: CommentBuckets,
    annotation: Option<Annotation>,
    modifiers: Vec<Modifier>,
}

impl<'a> DocBuild<'a> for Modifiers {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.annotation {
            result.push(n.build(b));
        }

        if !self.modifiers.is_empty() {
            let modifiers_doc = b.build_docs(&self.modifiers);
            result.push(b.concat(modifiers_doc));
            result.push(b.txt(" "));
        }
    }
}

impl Modifiers {
    pub fn new(node: Node) -> Self {
        assert_check(node, "modifiers");
        let mut modifiers = Self::default();

        for c in node.children_vec() {
            match c.kind() {
                "annotation" => {
                    modifiers.annotation = Some(Annotation {
                        name: c.cvalue_by_n("name", source_code()),
                    });
                }
                "modifier" => match c.first_c().kind() {
                    "public" => modifiers.modifiers.push(Modifier::Public),
                    _ => panic!("## unknown node: {} in Modifier", c.kind().red()),
                },
                "line_comment" | "block_comment" => continue,
                _ => panic!("## unknown node: {} in Modifiers", c.kind().red()),
            }
        }

        modifiers
    }
}

#[derive(Debug, Serialize)]
pub struct Annotation {
    pub name: String,
}

impl<'a> DocBuild<'a> for Annotation {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt(format!("@{}", self.name)));
        result.push(b.nl());
    }
}

#[derive(Debug, Serialize)]
pub struct ClassBody {
    pub declarations: Vec<ClassMember>,
}

impl ClassBody {
    pub fn new(node: Node) -> Self {
        assert_check(node, "class_body");
        let mut declarations: Vec<ClassMember> = Vec::new();

        for c in node.children_vec() {
            match c.kind() {
                "field_declaration" => {
                    declarations.push(ClassMember::Field(Box::new(FieldDeclaration::new(c))))
                }
                "class_declaration" => {
                    declarations.push(ClassMember::NestedClass(Box::new(ClassDeclaration::new(c))))
                }
                "block" => declarations.push(ClassMember::Block(Box::new(Block::new(c)))),
                "line_comment" | "block_comment" => continue,
                _ => panic!("## unknown node: {} in ClassBody ", c.kind().red()),
            }
        }

        Self { declarations }
    }
}

impl<'a> DocBuild<'a> for ClassBody {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        let member_docs = b.build_docs(&self.declarations);
        let body_doc = b.sep_multi_line(&member_docs, "");
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

impl<'a> DocBuild<'a> for FieldDeclaration {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        if let Some(ref n) = self.modifiers {
            result.push(n.build(b));
        }

        result.push(self.type_.build(b));
        result.push(b.txt(" "));

        let decl_docs = b.build_docs(&self.declarators);

        let declarators_doc = b.separated_choice(&decl_docs, ", ", ", ");
        result.push(declarators_doc);

        result.push(b.txt(";"));
    }
}

impl FieldDeclaration {
    pub fn new(node: Node) -> Self {
        assert_check(node, "field_declaration");
        let buckets = None;

        let modifiers = node.try_c_by_k("modifiers").map(|n| Modifiers::new(n));

        let type_node = node.c_by_n("type");
        let type_ = match type_node.kind() {
            "type_identifier" => {
                UnnanotatedType::Simple(SimpleType::Identifier(Identifier::new(type_node)))
            }
            _ => panic!(
                "## unknown node: {} in FieldDeclaration ",
                type_node.kind().red()
            ),
        };

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

#[derive(Debug, Serialize)]
pub struct VariableDeclarator {
    pub name: String,
    pub value: Option<VariableInitializer>,
}

impl<'a> DocBuild<'a> for VariableDeclarator {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt(&self.name));
        result.push(b.txt(" = "));
        if let Some(ref v) = self.value {
            result.push(v.build(b));
        }
    }
}

impl VariableDeclarator {
    pub fn new(node: Node) -> Self {
        assert_check(node, "variable_declarator");
        let name = node.cvalue_by_n("name", source_code());

        let value = node.try_c_by_n("value").map(|v| match v.kind() {
            //"array_initializer" => {
            //    VariableInitializer::ArrayInitializer(ArrayInitializer::new(v, source_code, indent))
            //}
            _ => VariableInitializer::Expression(Expression::Primary(
                PrimaryExpression::Identifier(Identifier {
                    value: v.value(source_code()),
                }),
            )),
        });

        Self { name, value }
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
    pub fn new(node: Node, indent: usize) -> Self {
        assert_check(node, "assignment_expression");

        let left = node.cvalue_by_n("left", source_code());
        let op = node.cvalue_by_n("operator", source_code());
        let right = node.cvalue_by_n("right", source_code());
        Self { left, op, right }
    }
}

#[derive(Debug, Serialize)]
pub struct Identifier {
    pub value: String,
}

impl Identifier {
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
        Block::default()
    }
}

impl<'a> DocBuild<'a> for Block {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("{"));

        if !self.statements.is_empty() {
            let statement_docs = b.build_docs(&self.statements);
            let block_doc = b.sep_multi_line(&statement_docs, "");
            result.push(block_doc);
        }
        result.push(b.nl());
        result.push(b.txt("}"));
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
            match c.kind() {
                "type_identifier" => {
                    interface
                        .types
                        .push(Type::Unnanotated(UnnanotatedType::Simple(
                            SimpleType::Identifier(Identifier::new(c)),
                        )))
                }
                _ => panic!("## unknown node: {} in ClassBody ", c.kind().red()),
            }
        }

        interface
    }
}

impl<'a> DocBuild<'a> for Interface {
    fn build_inner(&self, b: &'a DocBuilder<'a>, result: &mut Vec<DocRef<'a>>) {
        result.push(b.txt("implements"));
        result.push(b.softline());

        let types_doc = b.build_docs(&self.types);
        let doc = b.sep_single_line(&types_doc, ", ");
        result.push(doc);
    }
}
