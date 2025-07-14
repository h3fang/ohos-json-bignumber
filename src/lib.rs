mod bignumber;
mod error;
mod options;
mod parse;
mod stringify;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
