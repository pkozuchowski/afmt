use crate::context::FmtContext;
use crate::define_struct;
use tree_sitter::Node;

define_struct!(
    ClassDeclaration,
    FieldDeclaration,
    MethodDeclaration,
    EnumDeclaration,
    EnumConstant,
    EnumBody,
    Block,
    Statement,
    ExpressionStatement,
    DoStatement,
    WhileStatement,
    ForStatement,
    EnhancedForStatement,
    Value,
    SuperClass,
    Expression,
    ArrayAccess,
    PrimaryExpression,
    DmlExpression,
    DmlSecurityMode,
    DmlType,
    AssignmentExpression,
    LocalVariableDeclaration,
    VariableDeclarator,
    IfStatement,
    UpdateExpression,
    ParenthesizedExpression,
    Interfaces,
    LineComment,
    ReturnStatement,
    ArgumentList,
    TypeArguments,
    GenericType,
    ArrayInitializer,
    DimensionsExpr,
    ArrayType,
    MapInitializer,
    Annotation,
    AnnotationArgumentList,
    AnnotationKeyValue,
    Modifiers,
    ConstructorDeclaration,
    ConstructorBody,
    ExplicitConstructorInvocation,
    RunAsStatement,
    ScopedTypeIdentifier,
    ObjectCreationExpression,
    TryStatement,
    CatchClause,
    CatchFormalParameter,
    FinallyClause,
    FieldAccess,
    InstanceOfExpression,
    CastExpression,
    Boolean,
    TernaryExpression,
    MethodInvocation,
    AccessorList,
    AccessorDeclaration,
    QueryExpression,
    SoqlQuery,
    SoqlQueryBody,
    SoslQuery,
    BinaryExpression,
    UnaryExpression,
    ArrayCreationExpression,
    MapCreationExpression
);
