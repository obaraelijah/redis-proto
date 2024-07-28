use crate::server::process_command;
use num_traits::cast::ToPrimitive;
use std::{error::Error, sync::Arc};
use tokio::sync::mpsc::{Receiver, Sender};

use x9::ast::Expr;
use x9::ffi::{ForeignData, IntoX9Function, Variadic, X9Interpreter};

struct FFIError {
    reason: String,
}

impl FFIError {
    fn boxed(reason: String) -> Box<dyn Error + Send> {
        Box::new(Self { reason })
    }
}

impl std::fmt::Debug for FFIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.reason)
    }
}

impl std::fmt::Display for FFIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.reason)
    }
}

impl Error for FFIError {}
