//! Development-only resource resolution for a detached reload candidate.
use super::{IResourceNode, ResourceKey, ResourceValue};
use crate::styling::ThemeVariant;

#[cfg(all(debug_assertions, feature = "hot-reload"))]
mod development {
    use super::*;
    use std::{cell::RefCell, collections::HashMap, rc::Rc};
    type Factory = Rc<dyn Fn() -> Rc<dyn IResourceNode>>;
    type Previews = HashMap<usize, Factory>;
    crate::ferro_thread_local! {
        static CURRENT: RefCell<Option<Previews>> = const { RefCell::new(None) };
    }
    /// A UI-thread candidate scope. It never installs owners or raises resource
    /// notifications. Factories may recursively resolve other prepared providers.
    pub struct ResourcePreviewScope {
        previous: Option<Previews>,
    }
    impl Default for ResourcePreviewScope {
        fn default() -> Self {
            Self::new()
        }
    }
    impl ResourcePreviewScope {
        pub fn new() -> Self {
            Self {
                previous: CURRENT.with(|current| current.replace(Some(HashMap::new()))),
            }
        }
        /// Returns false when two adapters claim the same live contribution.
        pub fn insert(&self, original: &dyn IResourceNode, factory: Factory) -> bool {
            CURRENT.with(|current| {
                let mut current = current.borrow_mut();
                let entries = current.as_mut().expect("active resource preview scope");
                let key = original.reference_id() as usize;
                if entries.contains_key(&key) {
                    return false;
                }
                entries.insert(key, factory);
                true
            })
        }
    }
    impl Drop for ResourcePreviewScope {
        fn drop(&mut self) {
            let retired = CURRENT.try_with(|current| current.replace(self.previous.take()));
            // Resource destructors must not run under the registry's RefCell borrow.
            drop(retired);
        }
    }
    /// Overrides one detached candidate node while retaining the enclosing
    /// transaction's previews. Guards must be dropped in lexical scope order.
    pub struct ResourcePreviewGuard {
        key: usize,
        previous: Option<Factory>,
        created_scope: bool,
    }
    impl ResourcePreviewGuard {
        pub fn new(original: &dyn IResourceNode, factory: Factory) -> Self {
            CURRENT.with(|current| {
                let mut current = current.borrow_mut();
                let created_scope = current.is_none();
                let key = original.reference_id() as usize;
                let previous = current.get_or_insert_with(HashMap::new).insert(key, factory);
                Self { key, previous, created_scope }
            })
        }
    }
    impl Drop for ResourcePreviewGuard {
        fn drop(&mut self) {
            let retired = CURRENT.with(|current| {
                let mut current = current.borrow_mut();
                if self.created_scope {
                    return current.take().map(|entries| Box::new(entries) as Box<dyn std::any::Any>);
                }
                let entries = current.as_mut().expect("enclosing preview scope is active");
                let retired = match self.previous.take() {
                    Some(previous) => entries.insert(self.key, previous),
                    None => entries.remove(&self.key),
                };
                retired.map(|factory| Box::new(factory) as Box<dyn std::any::Any>)
            });
            drop(retired);
        }
    }
    pub(super) fn candidate(original: &dyn IResourceNode) -> Option<Rc<dyn IResourceNode>> {
        let factory = CURRENT.with(|current| {
            current
                .borrow()
                .as_ref()
                .and_then(|entries| entries.get(&(original.reference_id() as usize)))
                .cloned()
        });
        factory.map(|factory| factory())
    }
}
#[cfg(all(debug_assertions, feature = "hot-reload"))]
pub use development::{ResourcePreviewGuard, ResourcePreviewScope};

/// Resolves a resource against the active candidate, or the original provider.
#[inline]
pub fn lookup(
    node: &dyn IResourceNode,
    key: &ResourceKey,
    theme: Option<&ThemeVariant>,
) -> Option<ResourceValue> {
    #[cfg(all(debug_assertions, feature = "hot-reload"))]
    if let Some(candidate) = development::candidate(node) {
        return candidate.try_get_resource(key, theme);
    }
    node.try_get_resource(key, theme)
}
