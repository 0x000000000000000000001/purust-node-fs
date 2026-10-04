// Rust FFI for Node.FS.Aff. `nativeAffImpl` turns a `Node.FS.Sync` `Effect`
// into an Aff wait executed on Tokio's blocking pool (see
// `Purs_Effect_Aff::purust_aff_from_blocking`). The completion is an internal
// Aff event, not a user Effect callback, so it may bypass the microtask queue.
// The PureScript fallback (the previous `makeAff` dispatch through
// `Node.FS.Async`) is only used by the JavaScript oracle.
use std::rc::Rc;

pub fn Node_FS_Aff_nativeAffImpl(
    _fallback: purust_core::Func1<(), crate::UnknownType>,
    effect: crate::UnknownType,
) -> crate::UnknownType {
    Purs_Effect_Aff::purust_aff_from_blocking(effect)
}

// `writeTextFile` keeps the exact `Node.FS.Async` error text: the historical
// dispatch reported every `std::fs::write` failure, including the write step,
// as an "open" error. The blocking effect stays deferred and is replayed per
// Aff run by the generic helper; the encoding is ignored exactly like the
// Async and Sync implementations.
pub fn Node_FS_Aff_nativeAffWriteTextImpl(
    _fallback: purust_core::Func1<(), crate::UnknownType>,
    path: crate::UnknownType,
    text: String,
) -> crate::UnknownType {
    let effect = crate::Value::Func1(purust_core::Func1::Shared(Rc::new(
        move |_unit: crate::UnknownType| -> crate::UnknownType {
            let path = purust_core::purust_string_to_utf8_lossy(&path.unwrap_string());
            let bytes = purust_core::purust_string_to_utf8_lossy(&text).into_bytes();
            if let Err(error) = std::fs::write(&path, bytes) {
                Purs_Effect_Exception::purust_exception_raise(
                    Purs_Node_FS_Async::purust_fs_error("open", &path, error),
                );
            }
            crate::Value::Unit
        },
    )));
    Purs_Effect_Aff::purust_aff_from_blocking(effect)
}
