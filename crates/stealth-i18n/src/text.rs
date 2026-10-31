//! [`Text`]: a message made where its words are not known yet (the simulation, the campaign, the
//! server) and translated where it is shown, in the reader's language.

use std::collections::BTreeSet;

use fluent_syntax::ast::{Expression, InlineExpression, Pattern, PatternElement};
use serde::{Deserialize, Serialize};

use crate::{Args, Language, tr_args_in};

/// A message id and its arguments, or a text shown as it is (a name, a chat line, a message
/// saved before the translations).
///
/// Serialized as a plain string when raw, so that the old saves read back.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Text {
    Raw(String),
    Msg { id: String, args: Vec<(String, Arg)> },
}

/// An argument of a [`Text`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Arg {
    Str(String),
    Int(i64),
    /// Translated too (a unit's or an outcome's name).
    Text(Text),
}

impl Text {
    pub fn new(id: impl Into<String>) -> Self {
        Text::Msg { id: id.into(), args: Vec::new() }
    }

    pub fn raw(s: impl Into<String>) -> Self {
        Text::Raw(s.into())
    }

    /// Adds an argument (no effect on a raw text).
    pub fn arg(mut self, name: &str, v: impl Into<Arg>) -> Self {
        if let Text::Msg { args, .. } = &mut self {
            args.push((name.to_string(), v.into()));
        }
        self
    }

    /// The text in `l`.
    pub fn tr_in(&self, l: Language) -> String {
        match self {
            Text::Raw(s) => s.clone(),
            Text::Msg { id, args } => {
                let mut a = Args::new();
                for (k, v) in args {
                    match v {
                        Arg::Str(s) => a.set(k.as_str(), s.as_str()),
                        Arg::Int(n) => a.set(k.as_str(), *n),
                        Arg::Text(t) => a.set(k.as_str(), t.tr_in(l)),
                    }
                }
                tr_args_in(l, id, Some(&a))
            }
        }
    }

    /// The text in the player's language.
    pub fn tr(&self) -> String {
        self.tr_in(crate::language())
    }

    /// The text for the flight font: in the player's language if it is written in Latin
    /// letters, else in English.
    pub fn tr_flight(&self) -> String {
        self.tr_in(crate::flight_language())
    }

    /// Argument `name` as text (a translated one in the player's language); None for a raw
    /// text or without it.
    pub fn arg_text(&self, name: &str) -> Option<String> {
        let Text::Msg { args, .. } = self else { return None };
        args.iter().find(|(k, _)| k == name).map(|(_, v)| match v {
            Arg::Str(s) => s.clone(),
            Arg::Int(n) => n.to_string(),
            Arg::Text(t) => t.tr(),
        })
    }

    /// Whether this is the message `id`.
    pub fn is(&self, id: &str) -> bool {
        matches!(self, Text::Msg { id: i, .. } if i == id)
    }
}

impl Default for Text {
    fn default() -> Self {
        Text::Raw(String::new())
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.tr())
    }
}

impl From<&str> for Arg {
    fn from(s: &str) -> Self {
        Arg::Str(s.to_string())
    }
}

impl From<String> for Arg {
    fn from(s: String) -> Self {
        Arg::Str(s)
    }
}

impl From<&String> for Arg {
    fn from(s: &String) -> Self {
        Arg::Str(s.clone())
    }
}

impl From<Text> for Arg {
    fn from(t: Text) -> Self {
        Arg::Text(t)
    }
}

macro_rules! int_arg {
    ($($t:ty),*) => {$(
        impl From<$t> for Arg {
            fn from(n: $t) -> Self {
                Arg::Int(n as i64)
            }
        }
    )*};
}
int_arg!(i8, i16, i32, i64, u8, u16, u32, u64, usize, isize);

/// A [`Text`]: `text!("id")`, `text!("id", name = value, ...)`.
#[macro_export]
macro_rules! text {
    ($id:expr) => {
        $crate::Text::new($id)
    };
    ($id:expr, $($k:ident = $v:expr),+ $(,)?) => {
        $crate::Text::new($id)$(.arg(stringify!($k), $v))+
    };
}

/// The variables a pattern uses.
pub(crate) fn variables(p: &Pattern<&str>, out: &mut BTreeSet<String>) {
    for e in &p.elements {
        if let PatternElement::Placeable { expression } = e {
            expression_vars(expression, out);
        }
    }
}

fn expression_vars(e: &Expression<&str>, out: &mut BTreeSet<String>) {
    match e {
        Expression::Select { selector, variants } => {
            inline_vars(selector, out);
            for v in variants {
                variables(&v.value, out);
            }
        }
        Expression::Inline(i) => inline_vars(i, out),
    }
}

fn inline_vars(e: &InlineExpression<&str>, out: &mut BTreeSet<String>) {
    match e {
        InlineExpression::VariableReference { id } => {
            out.insert(id.name.to_string());
        }
        InlineExpression::FunctionReference { arguments, .. } => {
            for a in &arguments.positional {
                inline_vars(a, out);
            }
            for a in &arguments.named {
                inline_vars(&a.value, out);
            }
        }
        InlineExpression::Placeable { expression } => expression_vars(expression, out),
        _ => {}
    }
}
