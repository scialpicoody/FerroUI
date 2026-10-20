//! UI-thread source snapshots. A candidate snapshot is scoped until its revision commits.
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
pub type Sources = BTreeMap<String, String>;
ferroui_base::ferro_thread_local! { static CURRENT: RefCell<Rc<Sources>> = RefCell::new(Rc::new(Sources::new())); }
pub fn source(uri: &str) -> Option<String> {
    CURRENT
        .with(|slot| slot.borrow().get(uri).cloned())
        .or_else(|| published_source(uri))
}
/// Restores the prior snapshot on normal return and unwind. Never transfers UI objects across threads.
pub fn with_sources<T>(sources: Rc<Sources>, action: impl FnOnce() -> T) -> T {
    struct Restore(Option<Rc<Sources>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            CURRENT.with(|slot| *slot.borrow_mut() = self.0.take().unwrap());
        }
    }
    let _restore = Restore(Some(CURRENT.with(|slot| slot.replace(sources))));
    action()
}

/// Owns publication of one session's accepted documents. Scoped candidate sources always win.
/// A document can belong to only one live session on this UI thread.
pub struct SourceRegistration {
    id: u64,
}
ferroui_base::ferro_thread_local! {
    static PUBLISHED: RefCell<BTreeMap<u64, std::rc::Weak<RefCell<Rc<Sources>>>>> = RefCell::new(BTreeMap::new());
    static NEXT_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
}
impl SourceRegistration {
    pub fn new(sources: &Rc<RefCell<Rc<Sources>>>) -> Result<Self, String> {
        let id = NEXT_ID.with(|next| {
            let id = next.get();
            next.set(
                id.checked_add(1)
                    .expect("source registration IDs exhausted"),
            );
            id
        });
        let registration = Self { id };
        for uri in sources.borrow().keys() {
            registration.check_document(uri)?;
        }
        PUBLISHED.with(|published| {
            published.borrow_mut().insert(id, Rc::downgrade(sources));
        });
        Ok(registration)
    }
    pub fn check_document(&self, uri: &str) -> Result<(), String> {
        PUBLISHED.with(|published| {
            let mut published = published.borrow_mut();
            published.retain(|_, sources| sources.strong_count() != 0);
            for (id, sources) in published.iter() {
                if *id != self.id
                    && sources
                        .upgrade()
                        .is_some_and(|sources| sources.borrow().contains_key(uri))
                {
                    return Err(format!(
                        "Document already belongs to a reload session: {uri}"
                    ));
                }
            }
            Ok(())
        })
    }
}
impl Drop for SourceRegistration {
    fn drop(&mut self) {
        PUBLISHED.with(|published| {
            published.borrow_mut().remove(&self.id);
        });
    }
}
fn published_source(uri: &str) -> Option<String> {
    PUBLISHED.with(|published| {
        published
            .borrow()
            .values()
            .filter_map(std::rc::Weak::upgrade)
            .find_map(|sources| sources.borrow().get(uri).cloned())
    })
}
