//! Explicit ownership of literal styled-property assignments on preserved roots.
use crate::{PreparedReload, ReloadError};
use ferroui_base::{
    data::BindingPriority, diagnostics::FerroObjectDiagnosticExtensions, AnyValue, FerroObject,
    PropertyValue, Ref, StyledProperty,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub(crate) trait OwnedRootProperty {
    fn name(&self) -> &str;
    fn prepare(
        &self,
        root: &Ref<FerroObject>,
        candidate: &Ref<FerroObject>,
        before: &str,
        after: &str,
    ) -> Result<Option<Box<dyn PreparedReload>>, ReloadError>;
}
struct Property<T: PropertyValue> {
    property: &'static StyledProperty<T>,
    expected: Rc<RefCell<Result<Option<T>, ReloadError>>>,
    version: Rc<Cell<u64>>,
}
fn snapshot<T: PropertyValue>(
    root: &FerroObject,
    property: &'static StyledProperty<T>,
) -> Result<Option<T>, ReloadError> {
    let diagnostic = root.get_diagnostic(property);
    if root.has_local_value_binding(property)
        || diagnostic.priority() < BindingPriority::LocalValue
        || diagnostic.is_overridden_current_value()
    {
        return Err(ReloadError::RestartRequired(format!(
            "Root {} has a binding, animation or current-value override",
            property.name()
        )));
    }
    Ok((diagnostic.priority() == BindingPriority::LocalValue).then(|| root.get_value(property)))
}
fn equal<T: PropertyValue>(a: &Option<T>, b: &Option<T>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => (a as &dyn AnyValue).value_eq(b),
        (None, None) => true,
        _ => false,
    }
}
pub(crate) fn own<T: PropertyValue>(
    root: &FerroObject,
    property: &'static StyledProperty<T>,
) -> Box<dyn OwnedRootProperty> {
    Box::new(Property {
        property,
        version: Rc::new(Cell::new(root.local_write_version(property))),
        expected: Rc::new(RefCell::new(snapshot(root, property))),
    })
}
fn attribute(source: &str, name: &str) -> Result<Option<String>, ReloadError> {
    let xml = roxmltree::Document::parse(source)
        .map_err(|error| ReloadError::Failed(error.to_string()))?;
    Ok(xml.root_element().attribute(name).map(str::to_owned))
}
impl<T: PropertyValue> OwnedRootProperty for Property<T> {
    fn name(&self) -> &str {
        self.property.name()
    }
    fn prepare(
        &self,
        root: &Ref<FerroObject>,
        candidate: &Ref<FerroObject>,
        before: &str,
        after: &str,
    ) -> Result<Option<Box<dyn PreparedReload>>, ReloadError> {
        let old = attribute(before, self.name())?;
        let new = attribute(after, self.name())?;
        if old == new {
            return Ok(None);
        }
        if matches!(
            self.name(),
            "Content" | "DataContext" | "Name" | "Resources" | "Styles" | "Theme" | "Template"
        ) {
            return Err(ReloadError::RestartRequired(format!(
                "Root {} requires its lifecycle adapter",
                self.name()
            )));
        }
        if old
            .iter()
            .chain(new.iter())
            .any(|value| value.trim_start().starts_with('{'))
        {
            return Err(ReloadError::RestartRequired(format!(
                "Root {} markup extensions require a binding/resource adapter",
                self.name()
            )));
        }
        let current = snapshot(root, self.property)?;
        let expected = self.expected.borrow().clone()?;
        if root.local_write_version(self.property) != self.version.get()
            || !equal(&current, &expected)
            || (old.is_none() && current.is_some())
        {
            return Err(ReloadError::RestartRequired(format!(
                "Application changed owned root {}",
                self.name()
            )));
        }
        let value = if new.is_some() {
            snapshot(candidate, self.property)?
                .ok_or_else(|| {
                    ReloadError::RestartRequired(format!(
                        "Root {} did not produce a literal local value",
                        self.name()
                    ))
                })?
                .into()
        } else {
            None
        };
        Ok(Some(Box::new(Change {
            root: root.clone(),
            property: self.property,
            before: current,
            after: value,
            expected: self.expected.clone(),
            version: self.version.clone(),
            committed_version: 0,
            started: false,
        })))
    }
}
struct Change<T: PropertyValue> {
    root: Ref<FerroObject>,
    property: &'static StyledProperty<T>,
    before: Option<T>,
    after: Option<T>,
    expected: Rc<RefCell<Result<Option<T>, ReloadError>>>,
    version: Rc<Cell<u64>>,
    started: bool,
    committed_version: u64,
}
impl<T: PropertyValue> Change<T> {
    fn assign(&self, value: &Option<T>) {
        match value {
            Some(value) => self.root.set_value(self.property, value.clone()),
            None => self.root.clear_value(self.property),
        }
    }
}
impl<T: PropertyValue> PreparedReload for Change<T> {
    fn commit(&mut self) -> Result<(), ReloadError> {
        if self.root.local_write_version(self.property) != self.version.get()
            || !equal(&snapshot(&self.root, self.property)?, &self.before)
        {
            return Err(ReloadError::RestartRequired(format!(
                "Root {} changed after preparation",
                self.property.name()
            )));
        }
        self.started = true;
        self.assign(&self.after);
        self.committed_version = self.root.local_write_version(self.property);
        Ok(())
    }
    fn rollback(&mut self) {
        if self.started {
            self.assign(&self.before);
            self.version
                .set(self.root.local_write_version(self.property));
        }
    }
    fn finish(&mut self) {
        *self.expected.borrow_mut() = Ok(self.after.clone());
        self.version.set(self.committed_version);
    }
}
