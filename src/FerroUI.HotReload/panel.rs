//! Replacement of an explicitly owned child range in a stable panel.
use crate::{ComponentRefresh, PreparedReload, ReloadError, ReloadTarget};
use ferroui_base::{
    controls::{NameScope, NameScopeRef},
    BoxedValue, Ref, WeakRef,
};
use ferroui_controls::{Control, Panel};
use ferroui_markup_xaml::{
    source_overlay::Sources, FromXamlObject, RuntimeXamlLoaderConfiguration,
    RuntimeXamlLoaderDocument,
};
use std::{cell::RefCell, ops::Range, rc::Rc};

struct Owned {
    children: Vec<WeakRef<Control>>,
    index: usize,
}
/// Opt-in adapter for framework panels. Only `owned` children are replaced;
/// application children outside that range retain their identity and ordering.
/// The root namescope must belong to this XAML document. Application children
/// needing names must own separate namescopes. Root assignments and handlers still
/// need their own ownership adapter; constructors are never rerun on the live root.
pub struct PanelReloadTarget {
    root: WeakRef<Panel>,
    uri: String,
    baseline: Rc<RefCell<String>>,
    owned: Rc<RefCell<Owned>>,
    configuration: RuntimeXamlLoaderConfiguration,
    refresh: Option<Rc<dyn ComponentRefresh>>,
    properties: Vec<Box<dyn crate::root_properties::OwnedRootProperty>>,
}
impl PanelReloadTarget {
    pub fn new(
        root: &Ref<Panel>,
        owned: Range<usize>,
        uri: String,
        baseline: String,
        configuration: RuntimeXamlLoaderConfiguration,
    ) -> Result<Self, ReloadError> {
        let children = root.children().to_vec();
        let range = children
            .get(owned.clone())
            .ok_or_else(|| ReloadError::Failed("Invalid owned panel child range".into()))?;
        if root.is_items_host() {
            return Err(ReloadError::RestartRequired(
                "Items-host panels require an items adapter".into(),
            ));
        }
        Ok(Self {
            root: root.downgrade(),
            uri: crate::document_key(&uri),
            baseline: Rc::new(RefCell::new(baseline)),
            owned: Rc::new(RefCell::new(Owned {
                children: range.iter().map(Ref::downgrade).collect(),
                index: owned.start,
            })),
            configuration,
            refresh: None,
            properties: Vec::new(),
        })
    }
    pub fn with_refresh(mut self, refresh: Rc<dyn ComponentRefresh>) -> Self {
        self.refresh = Some(refresh);
        self
    }
    /// Claims an initially populated literal root attribute for reversible reload.
    /// See ComponentReloadTarget::with_owned_root_property for the ownership contract.
    pub fn with_owned_root_property<T: ferroui_base::PropertyValue>(
        mut self,
        property: &'static ferroui_base::StyledProperty<T>,
    ) -> Self {
        if let Some(root) = self.root.upgrade() {
            if !self
                .properties
                .iter()
                .any(|owned| owned.name() == property.name())
            {
                self.properties
                    .push(crate::root_properties::own(&root, property));
            }
        }
        self
    }
    pub fn is_alive(&self) -> bool {
        self.root.upgrade().is_some()
    }
}
struct Change {
    root: Ref<Panel>,
    original: Vec<Ref<Control>>,
    old: Vec<Ref<Control>>,
    new: Vec<Ref<Control>>,
    index: usize,
    old_scope: Option<NameScopeRef>,
    new_scope: Option<NameScopeRef>,
    baseline: Rc<RefCell<String>>,
    owned: Rc<RefCell<Owned>>,
    before: String,
    after: String,
    state: Rc<crate::state::ViewState>,
    refresh: Option<Box<dyn PreparedReload>>,
    properties: Vec<Box<dyn PreparedReload>>,
    started: bool,
}
impl PreparedReload for Change {
    fn commit(&mut self) -> Result<(), ReloadError> {
        if self.root.children().to_vec() != self.original {
            return Err(ReloadError::RestartRequired(
                "Panel children changed after preparation".into(),
            ));
        }
        for property in &mut self.properties {
            property.commit()?;
        }
        self.started = true;
        for child in &self.old {
            self.root.children().remove(child.clone());
        }
        self.root
            .children()
            .insert_range(self.index, self.new.clone());
        NameScope::set_name_scope(&self.root, self.new_scope.clone());
        if let Some(refresh) = &mut self.refresh {
            refresh.commit()?;
        }
        Ok(())
    }
    fn rollback(&mut self) {
        for property in self.properties.iter_mut().rev() {
            property.rollback();
        }
        if !self.started {
            return;
        }
        // Also handles a partially completed removal/insertion before a setter panics.
        for child in self.new.iter().chain(&self.old) {
            self.root.children().remove(child.clone());
        }
        self.root
            .children()
            .insert_range(self.index, self.old.clone());
        NameScope::set_name_scope(&self.root, self.old_scope.clone());
        if let Some(refresh) = &mut self.refresh {
            refresh.rollback();
        }
    }
    fn finish(&mut self) {
        for property in &mut self.properties {
            property.finish();
        }
        *self.baseline.borrow_mut() = self.after.clone();
        *self.owned.borrow_mut() = Owned {
            children: self.new.iter().map(Ref::downgrade).collect(),
            index: self.index,
        };
        if let Some(refresh) = &mut self.refresh {
            refresh.finish();
        }
        let root = self.root.downgrade();
        let scope = self.new_scope.clone();
        let state = self.state.clone();
        let before = self.before.clone();
        let after = self.after.clone();
        ferroui_base::threading::Dispatcher::current_dispatcher().post_local(
            move || {
                if let Some(root) = root.upgrade() {
                    if NameScope::get_name_scope(&root) == scope {
                        let value: BoxedValue = Rc::new(root);
                        state.restore(&value, &before, &after);
                    }
                }
            },
            ferroui_base::threading::DispatcherPriority::BACKGROUND,
        );
    }
}
impl ReloadTarget for PanelReloadTarget {
    fn documents(&self) -> Vec<String> {
        vec![self.uri.clone()]
    }
    fn prepare(&self, sources: Rc<Sources>) -> Result<Box<dyn PreparedReload>, ReloadError> {
        let root = self
            .root
            .upgrade()
            .ok_or_else(|| ReloadError::Failed("Panel closed".into()))?;
        if root.is_items_host() {
            return Err(ReloadError::RestartRequired(
                "Items-host panels require an items adapter".into(),
            ));
        }
        let original = root.children().to_vec();
        let owned = self.owned.borrow();
        let old = owned
            .children
            .iter()
            .map(|child| {
                child.upgrade().ok_or_else(|| {
                    ReloadError::RestartRequired("Owned panel child was removed".into())
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let index = old
            .first()
            .and_then(|first| original.iter().position(|child| child == first))
            .unwrap_or(owned.index);
        if original.get(index..index + old.len()) != Some(old.as_slice()) {
            return Err(ReloadError::RestartRequired(
                "Owned panel children are no longer contiguous".into(),
            ));
        }
        let source = sources
            .get(&self.uri)
            .ok_or_else(|| ReloadError::Failed("Missing panel source".into()))?;
        let before = self.baseline.borrow().clone();
        let prepared = crate::component::shadow_source(
            source,
            &before,
            &[
                "Panel",
                "StackPanel",
                "WrapPanel",
                "DockPanel",
                "Canvas",
                "Grid",
                "UniformGrid",
            ],
            &self
                .properties
                .iter()
                .map(|property| property.name())
                .collect::<Vec<_>>(),
        )?;
        let uri = ferroui_base::utilities::Uri::try_create(
            &self.uri,
            ferroui_base::utilities::UriKind::Absolute,
        )
        .ok_or_else(|| ReloadError::Failed("Invalid panel URI".into()))?;
        let candidate = ferroui_markup_xaml_loader::FerroRuntimeXamlLoader::load_document(
            RuntimeXamlLoaderDocument::with_base_uri(Some(uri), &prepared),
            Some(self.configuration.clone()),
        )
        .map_err(|error| ReloadError::Failed(error.to_string()))?;
        let candidate = Ref::<Panel>::from_xaml_object(&candidate)
            .ok_or_else(|| ReloadError::RestartRequired("Document is not a panel".into()))?;
        let mut properties = Vec::new();
        for property in &self.properties {
            if let Some(change) = property.prepare(
                &root.clone().upcast(),
                &candidate.clone().upcast(),
                &before,
                source,
            )? {
                properties.push(change);
            }
        }
        let new = candidate.children().to_vec();
        let new_scope = NameScope::get_name_scope(&candidate);
        candidate.children().clear();
        NameScope::set_name_scope(&candidate, None);
        let refresh = self
            .refresh
            .as_ref()
            .map(|refresh| refresh.prepare(&root.clone().upcast(), new_scope.as_ref()))
            .transpose()?;
        let value: BoxedValue = Rc::new(root.clone());
        Ok(Box::new(Change {
            state: Rc::new(crate::state::ViewState::capture(Some(&value), &before)),
            old_scope: NameScope::get_name_scope(&root),
            root,
            original,
            old,
            new,
            index,
            new_scope,
            baseline: self.baseline.clone(),
            owned: self.owned.clone(),
            before,
            after: source.clone(),
            refresh,
            properties,
            started: false,
        }))
    }
}
