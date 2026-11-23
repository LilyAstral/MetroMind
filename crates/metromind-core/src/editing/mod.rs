//! Every change to the map is an [`Edit`]. Applying one returns its inverse,
//! which is all undo/redo needs.

mod connect;
mod edit;
mod line_ops;
mod search;
mod station_ops;
mod undo;

pub use connect::{ConnectRefusal, branch, branch_new_line, connect, connect_new_line, disconnect};
pub use edit::{Edit, Membership};
pub use search::search;
pub use undo::{UndoStack, touch};
