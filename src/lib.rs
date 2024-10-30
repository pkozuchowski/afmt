mod accessor;
pub mod args;
pub mod config;
mod fmt_push;
mod macros;
mod node_structs;
mod notation;
mod print;
mod rewrite;
mod rich_def;
mod rich_macro;
mod rich_node;
mod route;
mod shape;
mod struct_def;
mod utility;
mod visit;

use config::Session;

pub fn format(session: Session) {
    session.format();
}
