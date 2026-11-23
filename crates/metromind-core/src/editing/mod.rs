//! Every change to the map is an [`Edit`]. Applying one returns its inverse,
//! which is all undo/redo needs.

mod edit;
mod undo;

pub use edit::{Edit, Membership};
pub use undo::{UndoStack, touch};
