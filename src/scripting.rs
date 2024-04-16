use crate::server::process_command;
use std::{error::Error, sync::Arc};
use slog::error;
use tokio::sync::mpsc::{Sender, Receiver};
use num_traits::cast::ToPrimitive;

use crate::startup::Config;
use crate::types::Dumpfile;
use crate::types::RedisValueRef;
use crate::{logger::LOGGER, types::StateStoreRef};
use x7::ffi::{ForeignData, IntoX7Function, Variadic, X7Interpreter};
use x7::symbols::Expr;

fn bytes_to_string(s: &[u8]) -> String {
    String::from_utf8_lossy(s).to_string()
}

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

impl  Error for FFIError {}

impl ForeignData for RedisValueRef {
    fn to_x7(&self) -> Result<Expr, Box<dyn std::error::Error + Send>> {
        let res = match self {
            RedisValueRef::BulkString(s) | RedisValueRef::SimpleString(s) => {
                Expr::String(bytes_to_string(s))
            }
            RedisValueRef::Error(e) => {
                return Err(FFIError::boxed(bytes_to_string(e)));
            }
            RedisValueRef::ErrorMsg(e) => {
                return Err(FFIError::boxed(bytes_to_string(e)));
            }
            RedisValueRef::Int(i) => Expr::Integer(*i),
            RedisValueRef::Array(a) => {
                Expr::Tuple(a.iter().map(|ele| ele.to_x7()).collect::<Result<_, _>>()?)
            }
            RedisValueRef::NullArray | RedisValueRef::NullBulkString => Expr::Nil,
        };
        Ok(res)
    }

    fn from_x7(expr: &Expr) -> Result<Self, Box<dyn std::error::Error + Send>> {
        let res =
            match expr {
                Expr::Nil => RedisValueRef::NullArray,
                Expr::Num(n) => RedisValueRef::Int(n.to_i64().ok_or_else(|| {
                    FFIError::boxed(format!("Failed to convert {} into an i64", n))
                })?),
                Expr::Integer(n) => RedisValueRef::Int(*n),
                Expr::String(s) => RedisValueRef::BulkString(s.clone().into()),
                Expr::Symbol(s) => RedisValueRef::BulkString(s.read().into()),
                Expr::List(l) | Expr::Tuple(l) | Expr::Quote(l) => RedisValueRef::Array(
                    l.iter()
                        .map(ForeignData::from_x7)
                        .collect::<Result<_, _>>()?,
                ),
                Expr::Bool(b) => RedisValueRef::BulkString(format!("{}", b).into()),
                bad_type => {
                    return Err(FFIError::boxed(format!(
                        "redis-proto cannot reason about this type: {:?}",
                        bad_type
                    )))
                }
            };
        Ok(res)
    }
}


#[allow(clippy::type_complexity)]
pub struct ScriptingBridge {
    prog_send: Sender<(
        Program,
        OneShotSender<Result<RedisValueRef, Box<dyn Error + Send>>>,
    )>,
}

impl ScriptingBridge {
    #[allow(clippy::type_complexity)]
    pub fn new(
        prog_send: Sender<(
            Program,
            OneShotSender<Result<RedisValueRef, Box<dyn Error + Send>>>,
        )>,
    ) -> Arc<Self> {
        let sb = Self { prog_send };
        Arc::new(sb)
    }

    pub async fn handle_script_cmd(&self, cmd: Program) -> RedisValueRef {
        let (sx, rx) = oneshot_channel();
        if let Err(e) = self.prog_send.send((cmd, sx)).await {
            error!(LOGGER, "Failed to send program: {}", e);
        }
        match rx.await {
            Ok(x7_result) => match x7_result {
                Ok(r) => r,
                Err(e) => RedisValueRef::Error(format!("{}", e).into()),
            },
            Err(e) => {
                RedisValueRef::Error(format!("Failed to receive a response from x7 {}", e).into())
            }
        }
    }
}

use tokio::sync::oneshot::{channel as oneshot_channel, Sender as OneShotSender};

use tokio::sync::oneshot::error::TryRecvError;
pub async fn handle_redis_cmd(
    mut cmd_recv: Receiver<(Vec<RedisValueRef>, OneShotSender<RedisValueRef>)>,
    state_store: StateStoreRef,
    dump_file: Dumpfile,
    scripting_engine: Arc<ScriptingBridge>,
) {
    todo!()
}

#[derive(Debug)]
pub enum Program {
    String(String),
    Function(String, Vec<RedisValueRef>),
}

