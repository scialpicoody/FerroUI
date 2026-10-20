//! Shared constructor-scoped and development populate hooks.
use crate::{FerroXamlLoader, XamlLoadException};
use ferroui_base::metadata::IServiceProvider;
use ferroui_base::{BoxedValue, TypeInfo};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type PopulateOverride =
    Rc<dyn Fn(Option<&Rc<dyn IServiceProvider>>, &BoxedValue) -> Result<(), XamlLoadException>>;

ferroui_base::ferro_thread_local! {
    #[cfg(any(feature = "runtime-populate", test))]
    static SCOPED: RefCell<HashMap<usize, PopulateOverride>> = RefCell::new(HashMap::new());
    #[cfg(all(feature = "hot-reload", debug_assertions))]
    static PERSISTENT: RefCell<HashMap<usize, PopulateOverride>> = RefCell::new(HashMap::new());
}

/// Restores the previous override even when construction unwinds. UI-thread only.
#[must_use]
pub struct PopulateOverrideGuard {
    key: usize,
    previous: Option<PopulateOverride>,
    _thread: std::marker::PhantomData<Rc<()>>,
}
impl Drop for PopulateOverrideGuard {
    fn drop(&mut self) {
        #[cfg(any(feature = "runtime-populate", test))]
        SCOPED.with(|map| {
            let mut map = map.borrow_mut();
            if let Some(previous) = self.previous.take() {
                map.insert(self.key, previous);
            } else {
                map.remove(&self.key);
            }
        });
    }
}
impl FerroXamlLoader {
    /// Constructor overrides take precedence over persistent development overrides.
    #[cfg(any(feature = "runtime-populate", test))]
    pub fn scoped_populate_override(
        class: &'static TypeInfo,
        populate: PopulateOverride,
    ) -> PopulateOverrideGuard {
        let key = class as *const TypeInfo as usize;
        let previous = SCOPED.with(|map| map.borrow_mut().insert(key, populate));
        PopulateOverrideGuard {
            key,
            previous,
            _thread: std::marker::PhantomData,
        }
    }

    #[cfg(all(feature = "hot-reload", debug_assertions))]
    pub fn set_populate_override(
        class: &'static TypeInfo,
        populate: Option<PopulateOverride>,
    ) -> Option<PopulateOverride> {
        PERSISTENT.with(|map| {
            let key = class as *const TypeInfo as usize;
            let mut map = map.borrow_mut();
            match populate {
                Some(populate) => map.insert(key, populate),
                None => map.remove(&key),
            }
        })
    }

    #[inline]
    pub fn populate_override(class: &'static TypeInfo) -> Option<PopulateOverride> {
        #[cfg(any(feature = "runtime-populate", test))]
        let key = class as *const TypeInfo as usize;
        #[cfg(any(feature = "runtime-populate", test))]
        let scoped = SCOPED.with(|map| map.borrow().get(&key).cloned());
        #[cfg(any(feature = "runtime-populate", test))]
        if scoped.is_some() {
            return scoped;
        }
        #[cfg(all(feature = "hot-reload", debug_assertions))]
        {
            return PERSISTENT.with(|map| map.borrow().get(&key).cloned());
        }
        #[cfg(not(all(feature = "hot-reload", debug_assertions)))]
        {
            let _ = class;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferroui_controls::Control;
    use std::cell::Cell;
    #[test]
    fn nested_override_restores_on_unwind() {
        let calls = Rc::new(Cell::new(0));
        let count = calls.clone();
        let _outer = FerroXamlLoader::scoped_populate_override(
            Control::TYPE,
            Rc::new(move |_, _| {
                count.set(count.get() + 1);
                Ok(())
            }),
        );
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _inner = FerroXamlLoader::scoped_populate_override(
                Control::TYPE,
                Rc::new(|_, _| panic!("inner")),
            );
            panic!("constructor");
        }));
        assert!(failed.is_err());
        FerroXamlLoader::populate_override(Control::TYPE).unwrap()(
            None,
            &(Rc::new(0_i32) as BoxedValue),
        )
        .unwrap();
        assert_eq!(calls.get(), 1);
        drop(_outer);
        assert!(FerroXamlLoader::populate_override(Control::TYPE).is_none());
    }
}

#[cfg(all(feature = "hot-reload", debug_assertions))]
ferroui_base::ferro_thread_local! {
    static LIVE: RefCell<HashMap<usize, Vec<ferroui_base::WeakRef<ferroui_base::FerroObject>>>> = RefCell::new(HashMap::new());
}
impl FerroXamlLoader {
    /// Weakly records a component without retaining it or changing its lifetime.
    #[inline]
    pub fn track_instance<T: ferroui_base::ObjectType>(
        class: &'static TypeInfo,
        root: &ferroui_base::Ref<T>,
    ) {
        #[cfg(all(feature = "hot-reload", debug_assertions))]
        LIVE.with(|live| {
            let root: ferroui_base::Ref<ferroui_base::FerroObject> = root.clone().upcast();
            let mut live = live.borrow_mut();
            let instances = live.entry(class as *const TypeInfo as usize).or_default();
            instances.retain(|weak| weak.upgrade().is_some());
            if !instances.iter().any(|weak| {
                weak.upgrade()
                    .is_some_and(|existing| existing.ptr_eq(&root))
            }) {
                instances.push(root.downgrade());
            }
        });
        #[cfg(not(all(feature = "hot-reload", debug_assertions)))]
        let _ = (class, root);
    }
    #[cfg(all(feature = "hot-reload", debug_assertions))]
    pub fn live_instances(
        class: &'static TypeInfo,
    ) -> Vec<ferroui_base::Ref<ferroui_base::FerroObject>> {
        LIVE.with(|live| {
            let mut live = live.borrow_mut();
            let instances = live.entry(class as *const TypeInfo as usize).or_default();
            instances.retain(|weak| weak.upgrade().is_some());
            instances.iter().filter_map(|weak| weak.upgrade()).collect()
        })
    }
}
