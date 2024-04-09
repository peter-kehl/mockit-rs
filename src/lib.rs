#![allow(type_alias_bounds)]
#![allow(incomplete_features)]
//#![feature(adt_const_params)]
//#![feature(macro_metavar_expr)]

// Never type `!` allows some zero cost abstractions (in release/pass-through, unmocked mode).
#![feature(never_type)]

mod macros;
mod read;
