use std::fmt;
use wain_ast::*;

#[cfg_attr(test, derive(Debug))]
pub enum ErrorKind {
    IndexOutOfBounds {
        idx: u32,
        upper: usize,
        what: &'static str,
    },
    MultipleReturnTypes(Vec<ValType>),
    TooFewFuncLocalsForParams {
        locals: usize,
        params: usize,
    },
    ParamTypeMismatchWithLocal {
        idx: usize,
        param: ValType,
        local: ValType,
    },
    UnknownImport {
        mod_name: String,
        name: String,
    },
    TypeMismatch {
        op: &'static str,
        expected: ValType,
        actual: ValType,
    },
    CtrlFrameEmpty {
        op: &'static str,
        frame_start: usize,
        idx_in_op_stack: usize,
    },
    LabelStackEmpty {
        op: &'static str,
    },
    SetImmutableGlobal {
        ty: ValType,
        idx: u32,
    },
    TooLargeAlign {
        align: u32,
        bits: u8,
    },
}

#[cfg_attr(test, derive(Debug))]
pub struct Error<'a> {
    kind: ErrorKind,
    source: &'a str,
    offset: usize,
}

struct Ordinal(usize);
impl fmt::Display for Ordinal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 % 10 {
            1 => write!(f, "{}st", self.0),
            2 => write!(f, "{}nd", self.0),
            3 => write!(f, "{}rd", self.0),
            _ => write!(f, "{}th", self.0),
        }
    }
}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;
        match &self.kind {
            IndexOutOfBounds { idx, upper, what } => write!(
                f,
                "{} index {} out of bounds 0 <= idx < {}",
                what, idx, upper
            )?,
            MultipleReturnTypes(tys) => {
                let ss = tys.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
                write!(
                    f,
                    "multiple return types are not allowed for now but got [{}]",
                    ss.join(", ")
                )?
            }
            TooFewFuncLocalsForParams { locals, params } => {
                write!(f, "function has {} params > {} locals", params, locals)?
            }
            ParamTypeMismatchWithLocal { idx, param, local } => write!(
                f,
                "type {} parameter {} does not match to type of respective local {}",
                param,
                Ordinal(*idx),
                local
            )?,
            UnknownImport { mod_name, name } => {
                if *mod_name != "env" {
                    write!(
                        f,
                        "unknown module name '{}'. valid module name is currently only 'env'",
                        mod_name
                    )?
                } else {
                    write!(
                        f,
                        "no exported name '{}' in module 'env'. currently only 'print' is exported",
                        name
                    )?
                }
            }
            TypeMismatch {
                op,
                expected,
                actual,
            } => write!(
                f,
                "type does not match at '{}': expected {} but got {}",
                op, expected, actual
            )?,
            CtrlFrameEmpty {
                op,
                frame_start,
                idx_in_op_stack: 0,
            } => write!(f, "operand stack cannot be empty at '{}' instruction while validating instruction sequence starting at offset {}", op, frame_start)?,
            CtrlFrameEmpty {
                op,
                frame_start,
                idx_in_op_stack,
            } => write!(
                f,
                "empty control frame cannot be empty at '{}' instruction. the frame started at byte offset {} and top of \
                 control frame is op_stack[{}]", op, frame_start, idx_in_op_stack)?,
            LabelStackEmpty { op } => write!(f, "label stack for control instructions is unexpectedly empty at '{}' instruction", op)?,
            SetImmutableGlobal{ ty, idx } => write!(f, "{} value cannot be set to immutable global variable {}", ty, idx)?,
            TooLargeAlign { align, bits } => write!(f, "align {} must not be larger than {}bits / 8", align, bits)?,
        }

        if self.offset == self.source.len() {
            write!(f, " caused at byte offset {} (end of input)", self.offset)
        } else {
            let source = &self.source[self.offset..];
            let end = source
                .find(['\n', '\r'].as_ref())
                .unwrap_or_else(|| source.len());
            write!(
                f,
                " caused at byte offset {}\n\n ... {}\n     ^\n     starts from here",
                self.offset,
                &source[..end],
            )
        }
    }
}

impl<'a> Error<'a> {
    pub(crate) fn new(kind: ErrorKind, offset: usize, source: &'a str) -> Box<Self> {
        Box::new(Self {
            kind,
            source,
            offset,
        })
    }
}

pub type Result<'a, T> = ::std::result::Result<T, Box<Error<'a>>>;
