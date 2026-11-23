//! The list draft behind the note list: the items as typed, the edits that
//! change them, and where focus should go next. Pure data, so it is tested
//! without a window.

use metromind_core::model::NoteItem;

/// The list as typed, which may differ from the saved list until it commits.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ListDraft {
    pub items: Vec<NoteItem>,
    /// Whether some item had focus last frame.
    pub(super) focused: bool,
    /// Row to focus next frame, with the cursor at the end of its text.
    pub(super) focus: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum RowAction {
    Tick(usize),
    Delete(usize),
    Insert(usize),
    Remove(usize),
    Move { from: usize, to: usize },
}

impl ListDraft {
    pub fn new(items: &[NoteItem]) -> Self {
        Self {
            items: items.to_vec(),
            ..Self::default()
        }
    }

    /// Mirror the saved list while no item is being typed in (e.g. after undo).
    pub fn sync(&mut self, saved: &[NoteItem]) {
        if !self.focused && self.items != saved {
            self.items = saved.to_vec();
        }
    }

    /// The list as it is saved: blank items dropped.
    pub fn cleaned(&self) -> Vec<NoteItem> {
        self.items
            .iter()
            .filter(|item| !item.text.trim().is_empty())
            .cloned()
            .collect()
    }

    pub(super) fn apply(&mut self, action: RowAction) {
        match action {
            RowAction::Tick(i) => self.items[i].done = !self.items[i].done,
            RowAction::Delete(i) => {
                self.items.remove(i);
            }
            RowAction::Insert(at) => {
                self.items.insert(
                    at,
                    NoteItem {
                        text: String::new(),
                        done: false,
                    },
                );
                self.focus = Some(at);
            }
            RowAction::Remove(i) => {
                // The last item stays: an empty list is shown by the Add button.
                if self.items.len() > 1 {
                    self.items.remove(i);
                    self.focus = Some(i.saturating_sub(1));
                }
            }
            RowAction::Move { from, to } => {
                self.items.swap(from, to);
                self.focus = Some(to);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(text: &str, done: bool) -> NoteItem {
        NoteItem {
            text: text.into(),
            done,
        }
    }

    fn draft(texts: &[&str]) -> ListDraft {
        ListDraft::new(&texts.iter().map(|t| item(t, false)).collect::<Vec<_>>())
    }

    #[test]
    fn blank_items_are_not_saved() {
        let d = draft(&["a", "  ", "", "b"]);
        assert_eq!(d.cleaned(), vec![item("a", false), item("b", false)]);
    }

    #[test]
    fn tick_toggles_and_keeps_text() {
        let mut d = draft(&["a"]);
        d.apply(RowAction::Tick(0));
        assert_eq!(d.items, vec![item("a", true)]);
    }

    #[test]
    fn insert_adds_an_empty_item_and_focuses_it() {
        let mut d = draft(&["a", "b"]);
        d.apply(RowAction::Insert(1));
        assert_eq!(d.items[1], item("", false));
        assert_eq!(d.focus, Some(1));
    }

    #[test]
    fn last_item_is_never_removed_by_backspace() {
        let mut d = draft(&[""]);
        d.apply(RowAction::Remove(0));
        assert_eq!(d.items.len(), 1);
        assert_eq!(d.focus, None);
    }

    #[test]
    fn remove_focuses_the_item_above() {
        let mut d = draft(&["a", ""]);
        d.apply(RowAction::Remove(1));
        assert_eq!(d.items, vec![item("a", false)]);
        assert_eq!(d.focus, Some(0));
    }

    #[test]
    fn move_swaps_and_follows_the_item() {
        let mut d = draft(&["a", "b"]);
        d.apply(RowAction::Move { from: 0, to: 1 });
        assert_eq!(d.items, vec![item("b", false), item("a", false)]);
        assert_eq!(d.focus, Some(1));
    }

    #[test]
    fn sync_only_while_not_typing() {
        let saved = vec![item("saved", false)];
        let mut d = draft(&["typing"]);
        d.focused = true;
        d.sync(&saved);
        assert_eq!(d.items, vec![item("typing", false)]);
        d.focused = false;
        d.sync(&saved);
        assert_eq!(d.items, saved);
    }
}
