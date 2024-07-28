use crate::server::process_command;
use num_traits::cast::ToPrimitive;
use std::{error::Error, sync::Arc};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::startup::Config;
use crate::types::DumpFile;
use crate::types::RedisValueRef;
use crate::{logger::LOGGER, types::StateStoreRef};
use x9::ast::Expr;
use x9::ffi::{ForeignData, IntoX9Function, Variadic, X9Interpreter};

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

impl Error for FFIError {}

impl ForeignData for RedisValueRef {
    fn from_x9(expr: &Expr) -> Result<Self, Box<dyn Error + Send>> {
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
                Expr::Tuple(a.iter().map(|ele| ele.to_x9()).collect::<Result<_, _>>()?)
            }
            RedisValueRef::NullArray | RedisValueRef::NullBulkString => Expr::Nil,
        };
        Ok(res)
    }

    fn to_x9(&self) -> Result<Expr, Box<dyn Error + Send>> {
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
                        .map(ForeignData::from_x9)
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
#[derive(Debug)]
pub enum Program {
    String(String),
    Function(String, Vec<RedisValueRef>),
}

pub struct ScriptingEngine {
    interpreter: X9Interpreter,
    #[allow(clippy::type_complexity)]
    prog_revc: Receiver<(
        Program,
        OneShotSender<Result<RedisValueRef, Box<dyn Error + Send>>>,
    )>,
    // prog_send: Sender<Result<RedisValueRef, Box<dyn Error + Send>>>,
    cmd_send: Arc<Sender<(Vec<RedisValueRef>, OneShotSender<RedisValueRef>)>>,
}

impl ScriptingEngine {
    pub fn new(
        prog_revc: Receiver<(
            Program,
            OneShotSender<Result<RedisValueRef, Box<dyn Error + Send>>>,
        )>,
        cmd_send: Sender<(Vec<RedisValueRef>, OneShotSender<RedisValueRef>)>,
        state_store: StateStoreRef,
        opts: &Config,
    ) -> Result<Self, Box<dyn Error>> {
        let res = Self {
            interpreter: X9Interpreter::new(),
            prog_revc,
            cmd_send: Arc::new(cmd_send),
        };
        res.setup_interpreter(state_store);
        res.load_scripts_dir(opts)?;
        Ok(res)
    }

    fn load_scripts_dir(&self, opts: &Config) -> Result<(), Box<dyn Error>> {
        if let Some(path) = &opts.scripts_dir {
            info!(LOGGER, "Loading scripts in {:?}", path);
            self.interpreter.load_lib_dir(path)
        } else {
            Ok(())
        }
    }

    fn add_redis_fn(&self) {
        let send_clone = self.cmd_send.clone();
        let send_fn = move |args: Variadic<RedisValueRef>| {
            let args = args.into_vec();
            let (sx, mut rx) = oneshot_channel();
            if let Err(e) = send_clone.blocking_send((args, sx)) {
                return Err(FFIError::boxed(format!(
                    "redis-proto failed to send the command: {}",
                    e
                )));
            }
            loop {
                match rx.try_recv() {
                    Ok(ret_value) => return Ok(ret_value),
                    Err(TryRecvError::Empty) => continue,
                    Err(TryRecvError::Closed) => {
                        return Err(FFIError::boxed(
                            "redix-proto failed to return a value!".into(),
                        ))
                    }
                }
            }
        };
        self.interpreter.add_function("redis", send_fn.to_x9_fn());
    }

    fn setup_interpreter(&self, state_store: StateStoreRef) {
        // "redis"
        self.add_redis_fn();
    }
}
