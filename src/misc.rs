use crate::types::{Count, Index, Key, RedisValueRef, ReturnValue, StateRef, StateStoreRef, Value};

op_variants! {
    MiscOps,
    Pong(),
    FlushAll(),
    FlushDB()
}

macro_rules! create_commands_list {
    ($($ops:ident),*) => {
        {
            let mut res = Vec::new();
            $(
                let tmp = $ops.iter().cloned().map(|s| s.into()).collect();
                res.push(ReturnValue::MultiStringRes(tmp));
            )*
            ReturnValue::Array(res)
        }
    };
}

/// Easily get all keys out of each passed type.
macro_rules! get_all_keys {
    ($state:expr, $($type:ident),*) => {
        {
            let mut all = Vec::new();
            $(
                all.extend($state.$type.iter().map(|r| r.key().clone()));
            )*
            all
        }
    }
}

lazy_static! {
    static ref ALL_COMMANDS: ReturnValue = {
        use crate::keys::OP_VARIANTS as KEY_VARIANTS;
        use crate::sets::OP_VARIANTS as SET_VARIANTS;
        use crate::lists::OP_VARIANTS as LIST_VARIANTS;
        use crate::hashes::OP_VARIANTS as HASH_VARIANTS;
        use crate::sorted_sets::OP_VARIANTS as ZSET_VARIANTS;
        use crate::bloom::OP_VARIANTS as BLOOM_VARIANTS;
        use crate::stack::OP_VARIANTS as STACK_VARIANTS;
        use crate::hyperloglog::OP_VARIANTS as HYPERLOGLOG_VARIANTS;
        create_commands_list!(
            KEY_VARIANTS,
            LIST_VARIANTS,
            HASH_VARIANTS,
            SET_VARIANTS,
            ZSET_VARIANTS,
            BLOOM_VARIANTS,
            STACK_VARIANTS,
            HYPERLOGLOG_VARIANTS,
            OP_VARIANTS // Misc variants
        )
    };
}

pub async fn misc_interact(
    misc_op: MiscOps,
    state: &mut StateRef,
    state_store: StateStoreRef,
) -> ReturnValue {
    match misc_op {
        MiscOps::Pong() => ReturnValue::StringRes(Value::from_static(b"PONG")),
        MiscOps::FlushAll() => {
            let clear = |state: &StateRef| {
                state.kv.clear();
                state.sets.clear();
                state.lists.clear();
                state.hashes.clear();
                state.zsets.clear();
                state.blooms.clear();
            };
            for state in state_store.states.iter_mut() {
                clear(&state);
            }
            // let state_guard = state_store.states.lock();
            // for state in state_guard.values() {
            //     clear(state);
            // }
            ReturnValue::Ok
        }
        MiscOps::FlushDB() => {
            *state = Default::default();
            ReturnValue::Ok
        }
    }
}