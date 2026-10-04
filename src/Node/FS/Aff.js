// JavaScript oracle: the previous Aff implementation (makeAff through
// Node.FS.Async) stays authoritative; the blocking native path is Rust-only.
export const nativeAffImpl = fallback => _effect => fallback();

// Same deferred fallback for the byte-exact writeTextFile path.
export const nativeAffWriteTextImpl = fallback => _path => _text => fallback();
