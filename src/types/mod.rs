mod check;

pub use check::TypeChecker;

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use crate::ast::{Block, TypeAnn};
use crate::span::Span;
use crate::symbols::SymbolIndex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Array(Box<Type>),
    Struct {
        name: String,
        fields: HashMap<String, Type>,
    },
    Enum {
        name: String,
        variants: HashMap<String, Vec<Type>>,
    },
    Option(Box<Type>),
    Result {
        ok: Box<Type>,
        err: Box<Type>,
    },
    Function {
        params: Vec<Type>,
        ret: Box<Type>,
    },
    Void,
    Error,
    TypeParam(String),
}

impl Type {
    pub fn from_ann(
        ann: &TypeAnn,
        structs: &HashMap<String, StructInfo>,
        enums: &HashMap<String, EnumInfo>,
    ) -> Type {
        match ann {
            TypeAnn::Int => Type::Int,
            TypeAnn::Float => Type::Float,
            TypeAnn::Bool => Type::Bool,
            TypeAnn::String => Type::String,
            TypeAnn::Array(inner) => {
                Type::Array(Box::new(Type::from_ann(inner, structs, enums)))
            }
            TypeAnn::Named(name) if name == "void" => Type::Void,
            TypeAnn::Named(name) => {
                if let Some(s) = structs.get(name) {
                    Type::Struct {
                        name: name.clone(),
                        fields: s.fields_map(),
                    }
                } else if let Some(e) = enums.get(name) {
                    Type::Enum {
                        name: name.clone(),
                        variants: e.variants.clone(),
                    }
                } else {
                    Type::Struct {
                        name: name.clone(),
                        fields: HashMap::new(),
                    }
                }
            }
            TypeAnn::Option(inner) => {
                Type::Option(Box::new(Type::from_ann(inner, structs, enums)))
            }
            TypeAnn::Result { ok, err } => Type::Result {
                ok: Box::new(Type::from_ann(ok, structs, enums)),
                err: Box::new(Type::from_ann(err, structs, enums)),
            },
        }
    }

    pub fn name(&self) -> String {
        match self {
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Array(inner) => format!("{}[]", inner.name()),
            Type::Struct { name, .. } => name.clone(),
            Type::Enum { name, .. } => name.clone(),
            Type::Option(inner) => format!("Option<{}>", inner.name()),
            Type::Result { ok, err } => format!("Result<{}, {}>", ok.name(), err.name()),
            Type::Function { .. } => "function".to_string(),
            Type::Void => "void".to_string(),
            Type::Error => "error".to_string(),
            Type::TypeParam(name) => name.clone(),
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Type::Error)
    }

    pub fn option_inner(&self) -> Option<&Type> {
        match self {
            Type::Option(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn result_inner(&self) -> Option<(&Type, &Type)> {
        match self {
            Type::Result { ok, err } => Some((ok, err)),
            _ => None,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<(String, Type)>,
    pub span: Span,
}

impl StructInfo {
    pub fn field_type(&self, name: &str) -> Option<&Type> {
        self.fields.iter().find(|(n, _)| n == name).map(|(_, t)| t)
    }

    pub fn fields_map(&self) -> HashMap<String, Type> {
        self.fields.iter().cloned().collect()
    }
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub variants: HashMap<String, Vec<Type>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TestInfo {
    pub name: String,
    pub body: Vec<TypedStmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypedProgram {
    pub functions: HashMap<String, FunctionInfo>,
    pub structs: HashMap<String, StructInfo>,
    pub enums: HashMap<String, EnumInfo>,
    pub tests: Vec<TestInfo>,
    pub top_level: Vec<TypedStmt>,
    pub symbols: SymbolIndex,
    pub source_file: PathBuf,
}

#[derive(Debug, Clone)]
pub struct GenericFunctionInfo {
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<(String, TypeAnn)>,
    pub ret_type: TypeAnn,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TraitInfo {
    pub name: String,
    pub methods: HashMap<String, (Vec<(String, Type)>, Type)>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret: Type,
    pub body: Vec<TypedStmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypedStmt {
    Let {
        name: String,
        ty: Type,
        mutable: bool,
        value: TypedExpr,
        span: Span,
    },
    Expr(TypedExpr),
    If {
        condition: TypedExpr,
        then_block: Vec<TypedStmt>,
        else_block: Option<Vec<TypedStmt>>,
    },
    While {
        condition: TypedExpr,
        body: Vec<TypedStmt>,
    },
    ForInt {
        var: String,
        start: i64,
        end: i64,
        body: Vec<TypedStmt>,
    },
    ForArray {
        var: String,
        array: TypedExpr,
        elem_ty: Type,
        body: Vec<TypedStmt>,
    },
    Return {
        value: Option<TypedExpr>,
    },
    Match {
        scrutinee: TypedExpr,
        arms: Vec<TypedMatchArm>,
        ty: Type,
    },
    Break,
    Continue,
    Block(Vec<TypedStmt>),
}

#[derive(Debug, Clone)]
pub struct TypedMatchArm {
    pub pattern: TypedPattern,
    pub body: Vec<TypedStmt>,
}

#[derive(Debug, Clone)]
pub enum TypedPattern {
    Wildcard,
    Literal(TypedExpr),
    Variant {
        enum_name: String,
        variant: String,
        payload_types: Vec<Type>,
        bindings: Vec<String>,
    },
    Struct {
        struct_name: String,
        fields: Vec<(String, String, Type)>,
    },
}

#[derive(Debug, Clone)]
pub enum TypedExpr {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Ident {
        name: String,
        ty: Type,
        span: Span,
    },
    Binary {
        op: crate::ast::BinOp,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
        ty: Type,
    },
    Unary {
        op: crate::ast::UnOp,
        expr: Box<TypedExpr>,
        ty: Type,
    },
    Call {
        name: String,
        args: Vec<TypedExpr>,
        ty: Type,
    },
    Index {
        target: Box<TypedExpr>,
        index: Box<TypedExpr>,
        ty: Type,
    },
    Field {
        target: Box<TypedExpr>,
        field: String,
        ty: Type,
    },
    Array {
        elements: Vec<TypedExpr>,
        ty: Type,
    },
    StructLit {
        name: String,
        fields: Vec<(String, TypedExpr)>,
        ty: Type,
    },
    Variant {
        enum_name: String,
        variant: String,
        payload: Vec<TypedExpr>,
        ty: Type,
    },
    Assign {
        name: String,
        value: Box<TypedExpr>,
        ty: Type,
    },
    Match {
        scrutinee: Box<TypedExpr>,
        arms: Vec<TypedMatchArm>,
        ty: Type,
    },
}

impl TypedExpr {
    pub fn ty(&self) -> Type {
        match self {
            TypedExpr::Int(_) => Type::Int,
            TypedExpr::Float(_) => Type::Float,
            TypedExpr::Bool(_) => Type::Bool,
            TypedExpr::String(_) => Type::String,
            TypedExpr::Ident { ty, .. }
            | TypedExpr::Binary { ty, .. }
            | TypedExpr::Unary { ty, .. }
            | TypedExpr::Call { ty, .. }
            | TypedExpr::Index { ty, .. }
            | TypedExpr::Field { ty, .. }
            | TypedExpr::Array { ty, .. }
            | TypedExpr::StructLit { ty, .. }
            | TypedExpr::Variant { ty, .. }
            | TypedExpr::Assign { ty, .. }
            | TypedExpr::Match { ty, .. } => ty.clone(),
        }
    }
}
