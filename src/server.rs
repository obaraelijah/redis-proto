use slog::{debug, error};

use crate::database::save_state;
use crate::misc::misc_interact;
use crate::ops::{op_interact, Ops};
/// Server launch file. Starts the services to make redis-proto work.
use crate::{logger::LOGGER, types::StateRef};
use crate::asyncresp::RespParser;
use crate::{
    ops::translate,
    startup::Config,
    types::{Dumpfile, RedisValueRef, ReturnValue, StateStoreRef},
};
use std::sync::atomic::Ordering;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Decoder;
use futures::{SinkExt, StreamExt};

fn incr_and_save_if_required(state: StateStoreRef, dump_file: Dumpfile) {
    state.commands_ran_since_save.fetch_add(1, Ordering::SeqCst);
    let should_save = state.commands_ran_since_save.compare_exchange(
        state.commands_threshold,
        0,
        Ordering::SeqCst,
        Ordering::SeqCst,
    );
    if should_save.is_ok() {
        let state_clone = state;
        let dump_file_clone = dump_file;
        tokio::spawn(async {
            save_state(state_clone, dump_file_clone);
        });
    }
}

pub async fn process_command(
    state: &mut StateRef,
    state_store: StateStoreRef,
    dump_file: Dumpfile,
    redis_value: RedisValueRef,
) -> RedisValueRef {
    match translate(redis_value, state_store.clone()) {
        Ok(op) => {
            debug!(LOGGER, "running op {:?}", op.clone());
            // Step 1: Execute the operation the operation (from translate above)
            let res: ReturnValue = match op {
                Ops::Misc(op) => {
                    misc_interact(op, state, state_store.clone()).await
                }
                _ => op_interact(op, state.clone()).await,
            };
            // Step 2: Update commands_ran_since_save counter, and save if necessary
            if !state_store.memory_only {
                incr_and_save_if_required(state_store.clone(), dump_file.clone());
            }
            // Step 3: Finally Return
            res.into()
        }
        Err(e) => RedisValueRef::from(e),
    }
}

/// Spawn a RESP handler for the given socket.
///
/// This will synchronously process requests / responses for this
/// connection only. Other connections will be spread across the
/// thread pool.
async fn process(
    socket: TcpStream,
    state_store: StateStoreRef,
    dump_file: Dumpfile,
) {
    tokio::spawn(async move {
        let mut state = state_store.get_default();
        let mut transport = RespParser::default().framed(socket);
        while let Some(redis_value) = transport.next().await {
            if let Err(e) = redis_value {
                error!(LOGGER, "Error recieving redis value {:?}", e);
                continue;
            }
            let res = process_command(
                &mut state,
                state_store.clone(),
                dump_file.clone(),
                redis_value.unwrap(),
            )
            .await;
            // let res = match translate(redis_value.unwrap()) {
            //     Ok(op) => {
            //         debug!(LOGGER, "running op {:?}", op.clone());
            //         // Step 1: Execute the operation the operation (from translate above)
            //         let res: ReturnValue = match op {
            //             Ops::Misc(op) => {
            //                 misc_interact(
            //                     op,
            //                     &mut state,
            //                     state_store.clone(),
            //                     scripting_bridge.clone(),
            //                 )
            //                 .await
            //             }
            //             _ => op_interact(op, state.clone()).await,
            //         };
            //         // Step 2: Update commands_ran_since_save counter, and save if necessary
            //         if !state_store.memory_only {
            //             incr_and_save_if_required(state_store.clone(), dump_file.clone());
            //         }
            //         // Step 3: Finally Return
            //         res.into()
            //     }
            //     Err(e) => RedisValueRef::from(e),
            // };
            if let Err(e) = transport.send(res).await {
                error!(LOGGER, "Failed to send data to client! {:?}", e)
            };
        }
    });
}

/// The listener for redis-proto. Accepts connections and spawns handlers.
pub async fn socket_listener(
    state_store: StateStoreRef,
    dump_file: Dumpfile,
    config: Config,
) {
    todo!()
}