//! Small helper shared by every contract that exposes a `list_*` query, so
//! pagination bounds-checking isn't reimplemented three times.

use soroban_sdk::{Env, IntoVal, TryFromVal, Val, Vec};

/// Upper bound on the number of items any single query may return, regardless
/// of the `limit` the caller asked for. Protects contracts from being made to
/// do unbounded work (and callers from unbounded resource fees) in a single
/// invocation.
pub const MAX_PAGE_SIZE: u32 = 50;

/// Clamps a caller-requested page size to `(0, MAX_PAGE_SIZE]`.
///
/// A requested `limit` of `0` is treated as "use the default page size"
/// rather than "return nothing", which is almost always what a caller means.
pub fn clamp_limit(limit: u32) -> u32 {
    if limit == 0 || limit > MAX_PAGE_SIZE {
        MAX_PAGE_SIZE
    } else {
        limit
    }
}

/// Returns the `[offset, offset + clamp_limit(limit))` slice of `items` as a
/// new `Vec`. Never panics: an out-of-range `offset` simply yields an empty
/// result.
pub fn paginate<T>(env: &Env, items: &Vec<T>, offset: u32, limit: u32) -> Vec<T>
where
    T: IntoVal<Env, Val> + TryFromVal<Env, Val>,
{
    let limit = clamp_limit(limit);
    let len = items.len();
    let mut out = Vec::new(env);
    let mut i = offset;
    while i < len && (i - offset) < limit {
        out.push_back(items.get_unchecked(i));
        i += 1;
    }
    out
}
