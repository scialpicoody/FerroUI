//! Opt-in adapter for declarative class components with a stable content root.
use crate::{PreparedReload, ReloadError, ReloadTarget};
use ferroui_base::{
    controls::{NameScope, NameScopeRef},
    BoxedValue, Ref, StyledElement, WeakRef,
};
use ferroui_controls::{ContentControl, ContentPage};
use ferroui_markup_xaml::{
    source_overlay::Sources, FromXamlObject, RuntimeXamlLoaderConfiguration,
    RuntimeXamlLoaderDocument,
};
use std::{cell::RefCell, rc::Rc};

enum Root {
    Control(WeakRef<ContentControl>),
    Page(WeakRef<ContentPage>),
}
#[derive(Clone)]
enum LiveRoot {
    Control(Ref<ContentControl>),
    Page(Ref<ContentPage>),
}
impl Root {
    fn upgrade(&self) -> Option<LiveRoot> {
        match self {
            Self::Control(root) => root.upgrade().map(LiveRoot::Control),
            Self::Page(root) => root.upgrade().map(LiveRoot::Page),
        }
    }
}
impl LiveRoot {
    fn element(&self) -> Ref<StyledElement> {
        match self {
            Self::Control(root) => root.clone().upcast(),
            Self::Page(root) => root.clone().upcast(),
        }
    }
    fn content(&self) -> Option<BoxedValue> {
        match self {
            Self::Control(root) => root.content(),
            Self::Page(root) => root.content(),
        }
    }
    fn set_content(&self, value: Option<BoxedValue>) {
        match self {
            Self::Control(root) => root.set_content(value),
            Self::Page(root) => root.set_content(value),
        }
    }
}
/// Prepares application reference/subscription changes without mutating the live component.
/// Commit runs after content and namescope replacement. Rollback runs after restoring the
/// previous graph; finish runs only after every target commits. Retain old subscriptions
/// until finish, dispose new subscriptions on rollback, and never rerun constructors.
/// As with ReloadTarget, the application must provide reversible, non-panicking cleanup.
pub trait ComponentRefresh {
    fn prepare(
        &self,
        root: &Ref<StyledElement>,
        scope: Option<&NameScopeRef>,
    ) -> Result<Box<dyn PreparedReload>, ReloadError>;
}
/// Explicitly opt in for declarative content. Use `with_refresh` for cached references and
/// application child subscriptions. The original component is never reconstructed. Root
/// assignments can be claimed with the literal, static-resource and binding ownership
/// methods. Inline resources/styles and XAML handlers need their own adapter.
pub struct ComponentReloadTarget {
    root: Root,
    uri: String,
    dependencies: Vec<String>,
    baseline: Rc<RefCell<String>>,
    configuration: RuntimeXamlLoaderConfiguration,
    refresh: Option<Rc<dyn ComponentRefresh>>,
    properties: Vec<Box<dyn crate::root_properties::OwnedRootProperty>>,
}
impl ComponentReloadTarget {
    /// Declares resources/styles outside the document's inline includes, such as
    /// application resources consumed through StaticResource. Their revisions are
    /// prepared and committed in the same transaction as this target.
    pub fn with_dependencies(mut self, dependencies: impl IntoIterator<Item = String>) -> Self {
        self.dependencies = dependencies
            .into_iter()
            .map(|uri| crate::document_key(&uri))
            .collect();
        self
    }

    pub fn content_control(
        root: &Ref<ContentControl>,
        uri: String,
        baseline: String,
        configuration: RuntimeXamlLoaderConfiguration,
    ) -> Self {
        Self {
            root: Root::Control(root.downgrade()),
            uri: crate::document_key(&uri),
            dependencies: Vec::new(),
            baseline: Rc::new(RefCell::new(baseline)),
            configuration,
            refresh: None,
            properties: Vec::new(),
        }
    }
    pub fn content_page(
        root: &Ref<ContentPage>,
        uri: String,
        baseline: String,
        configuration: RuntimeXamlLoaderConfiguration,
    ) -> Self {
        Self {
            root: Root::Page(root.downgrade()),
            uri: crate::document_key(&uri),
            dependencies: Vec::new(),
            baseline: Rc::new(RefCell::new(baseline)),
            configuration,
            refresh: None,
            properties: Vec::new(),
        }
    }
    pub fn with_refresh(mut self, refresh: Rc<dyn ComponentRefresh>) -> Self {
        self.refresh = Some(refresh);
        self
    }
    /// Claims XAML ownership of this literal root attribute. Call immediately after
    /// initial population, before application overrides. Binding/animation changes
    /// and externally changed values require restart. Custom property callbacks must
    /// support reversible assignment; no arbitrary user side effects are rolled back.
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
                    .push(crate::root_properties::own(&root.element(), property));
            }
        }
        self
    }
    /// Claims a root literal or StaticResource assignment, including Theme/Template.
    /// Re-resolves the candidate value on dependency revisions even when the markup
    /// attribute is unchanged. Call immediately after initial population. Dynamic
    /// resources, bindings and application overrides require another adapter or restart.
    /// Property callbacks must support reversible assignment.
    pub fn with_owned_root_static_resource_property<T: ferroui_base::PropertyValue>(
        mut self,
        property: &'static ferroui_base::StyledProperty<T>,
    ) -> Self {
        if let Some(root) = self.root.upgrade() {
            self.properties
                .retain(|owned| owned.name() != property.name());
            self.properties
                .push(crate::root_properties::own_static_resource(
                    &root.element(),
                    property,
                ));
        }
        self
    }
    pub fn is_alive(&self) -> bool {
        self.root.upgrade().is_some()
    }

    /// Claims a root literal or synchronous reflection/compiled binding. Register
    /// immediately after population, before application assignments. Preparation
    /// defers the candidate root binding and recreates it for the preserved root;
    /// its new namescope belongs to the candidate children. Custom binding kinds,
    /// delayed/asynchronous update modes and lifecycle properties need an adapter.
    pub fn with_owned_root_binding_property<T: ferroui_base::PropertyValue>(
        mut self,
        property: &'static ferroui_base::StyledProperty<T>,
    ) -> Self {
        if let Some(root) = self.root.upgrade() {
            self.properties.retain(|owned| owned.name() != property.name());
            self.properties.push(crate::root_properties::own_binding(&root.element(), property));
        }
        self
    }
}
pub(crate) fn shadow_source(
    source: &str,
    baseline: &str,
    supported: &[&str],
    content_member: &str,
    owned_properties: &[&str],
) -> Result<String, ReloadError> {
    const XAML: &str = "http://schemas.microsoft.com/winfx/2006/xaml";
    let parse = |text: &str| crate::DocumentContract::parse(text);
    if !parse(source)?.events.is_empty() {
        return Err(ReloadError::RestartRequired(
            "Component handlers require an explicit refresh adapter".into(),
        ));
    }
    let current =
        roxmltree::Document::parse(source).map_err(|e| ReloadError::Failed(e.to_string()))?;
    let before =
        roxmltree::Document::parse(baseline).map_err(|e| ReloadError::Failed(e.to_string()))?;
    let root = current.root_element();
    let old = before.root_element();
    let attributes = |node: roxmltree::Node<'_, '_>| {
        let mut attrs: Vec<_> = node
            .attributes()
            .filter(|a| a.namespace().is_some() || !owned_properties.contains(&a.name()))
            .map(|a| {
                (
                    a.namespace().unwrap_or("").to_owned(),
                    a.name().to_owned(),
                    a.value().to_owned(),
                )
            })
            .collect();
        attrs.sort();
        attrs
    };
    if root.tag_name() != old.tag_name() || attributes(root) != attributes(old) {
        return Err(ReloadError::RestartRequired(
            "Root assignment changes require an ownership-aware adapter".into(),
        ));
    }
    if root.attribute("Name").is_some()
        || root.attribute((XAML, "Name")).is_some()
        || root.attribute((crate::KEY_NAMESPACE, "Key")).is_some()
    {
        return Err(ReloadError::RestartRequired(
            "Named component roots require an explicit namescope adapter".into(),
        ));
    }
    for node in root.children().filter(|node| node.is_element()) {
        if let Some((owner, member)) = node.tag_name().name().split_once('.') {
            // Explicit Content/Children syntax owns the same graph as implicit
            // content. Other root property elements still need their own adapter.
            if member == content_member
                && supported.contains(&owner)
                && node.tag_name().namespace() == Some("https://github.com/ferroui")
                && node.attributes().len() == 0
            {
                continue;
            }
            return Err(ReloadError::RestartRequired(
                "Root property-element assignments require an ownership-aware adapter".into(),
            ));
        }
    }
    // Removing x:Class constructs only the framework root declared by the document. This
    // adapter deliberately never runs the application class constructor a second time.
    if !supported.contains(&root.tag_name().name())
        || root.tag_name().namespace() != Some("https://github.com/ferroui")
    {
        return Err(ReloadError::RestartRequired(
            "Unsupported component root".into(),
        ));
    }
    let mut result = source.to_owned();
    if let Some(class) = root
        .attributes()
        .find(|a| a.namespace() == Some(XAML) && a.name() == "Class")
    {
        result.replace_range(class.range(), "");
    }
    crate::identity::runtime_source(&result)
}
struct Change {
    root: LiveRoot,
    old_content: Option<BoxedValue>,
    new_content: Option<BoxedValue>,
    old_scope: Option<NameScopeRef>,
    new_scope: Option<NameScopeRef>,
    state: Rc<crate::state::ViewState>,
    before: String,
    after: String,
    baseline: Rc<RefCell<String>>,
    refresh: Option<Box<dyn PreparedReload>>,
    properties: Vec<Box<dyn PreparedReload>>,
    detached_context: Option<crate::detached_context::DetachedContext>,
}
impl PreparedReload for Change {
    fn commit(&mut self) -> Result<(), ReloadError> {
        for property in &mut self.properties {
            property.commit()?;
        }
        self.detached_context = crate::detached_context::DetachedContext::freeze(&self.old_content);
        self.root.set_content(self.new_content.clone());
        NameScope::set_name_scope(&self.root.element(), self.new_scope.clone());
        if let Some(refresh) = &mut self.refresh {
            refresh.commit()?;
        }
        Ok(())
    }
    fn rollback(&mut self) {
        for property in self.properties.iter_mut().rev() {
            property.rollback();
        }
        let _retired = crate::detached_context::DetachedContext::freeze(&self.new_content);
        self.root.set_content(self.old_content.clone());
        if let Some(context) = self.detached_context.take() {
            context.restore();
        }
        NameScope::set_name_scope(&self.root.element(), self.old_scope.clone());
        if let Some(refresh) = &mut self.refresh {
            refresh.rollback();
        }
    }
    fn finish(&mut self) {
        for property in &mut self.properties {
            property.finish();
        }
        *self.baseline.borrow_mut() = self.after.clone();
        if let Some(refresh) = &mut self.refresh {
            refresh.finish();
        }
        let root = self.root.element().downgrade();
        let content = self.new_content.clone();
        let scope = self.new_scope.clone();
        let state = self.state.clone();
        let before = self.before.clone();
        let after = self.after.clone();
        ferroui_base::threading::Dispatcher::current_dispatcher().post_local(
            move || {
                if let (Some(root), Some(content)) = (root.upgrade(), content) {
                    if NameScope::get_name_scope(&root) == scope {
                        state.restore(&content, &before, &after);
                    }
                }
            },
            ferroui_base::threading::DispatcherPriority::BACKGROUND,
        );
    }
}
impl ReloadTarget for ComponentReloadTarget {
    fn documents(&self) -> Vec<String> {
        let mut documents = self.dependencies.clone();
        documents.push(self.uri.clone());
        documents
    }
    fn prepare(&self, sources: Rc<Sources>) -> Result<Box<dyn PreparedReload>, ReloadError> {
        let root = self
            .root
            .upgrade()
            .ok_or_else(|| ReloadError::Failed("Component closed".into()))?;
        let source = sources
            .get(&self.uri)
            .ok_or_else(|| ReloadError::Failed("Missing component source".into()))?;
        let before = self.baseline.borrow().clone();
        let prepared = shadow_source(
            source,
            &before,
            &["ContentControl", "UserControl", "ContentPage"],
            "Content",
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
        .ok_or_else(|| ReloadError::Failed("Invalid component URI".into()))?;
        let bindings = ferroui_markup_xaml_loader::binding_capture::BindingCaptureScope::new(
            self.uri.clone(),
            self.properties.iter().filter_map(|property| property.binding_property()),
        );
        let candidate = ferroui_markup_xaml_loader::FerroRuntimeXamlLoader::load_document(
            RuntimeXamlLoaderDocument::with_base_uri(Some(uri), &prepared),
            Some(self.configuration.clone()),
        )
        .map_err(|e| ReloadError::Failed(e.to_string()))?;
        let candidate = match &root {
            LiveRoot::Control(_) => {
                Ref::<ContentControl>::from_xaml_object(&candidate).map(LiveRoot::Control)
            }
            LiveRoot::Page(_) => {
                Ref::<ContentPage>::from_xaml_object(&candidate).map(LiveRoot::Page)
            }
        }
        .ok_or_else(|| ReloadError::RestartRequired("Component content kind changed".into()))?;
        let mut properties = Vec::new();
        for property in &self.properties {
            if let Some(change) = property.prepare(
                &root.element().upcast(),
                &candidate.element().upcast(),
                &before,
                source,
                &bindings,
            )? {
                properties.push(change);
            }
        }
        let new_content = candidate.content();
        let new_scope = NameScope::get_name_scope(&candidate.element());
        candidate.set_content(None);
        NameScope::set_name_scope(&candidate.element(), None);
        let refresh = self
            .refresh
            .as_ref()
            .map(|refresh| refresh.prepare(&root.element(), new_scope.as_ref()))
            .transpose()?;
        Ok(Box::new(Change {
            old_content: root.content(),
            state: Rc::new(crate::state::ViewState::capture(
                root.content().as_ref(),
                &before,
            )),
            old_scope: NameScope::get_name_scope(&root.element()),
            root,
            new_content,
            new_scope,
            before,
            after: source.clone(),
            baseline: self.baseline.clone(),
            refresh,
            properties,
            detached_context: None,
        }))
    }
}
