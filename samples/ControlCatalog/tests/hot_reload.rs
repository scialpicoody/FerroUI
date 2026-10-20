//! Reload integration with application-owned view models and shared selection.
use ferroui_base::{threading::Dispatcher, Ref};
use ferroui_controls::{selection::ISelectionModel, ContentPage, Control, Window};
use ferroui_hot_reload::{
    ComponentReloadTarget, HotReloadSession, ReloadError, ReloadStatus, ReloadTarget,
};
use ferroui_markup_xaml::RuntimeXamlLoaderConfiguration;
use std::rc::Rc;

#[test]
fn catalog_root_resources_reload_without_replacing_page_dictionary_or_application_keys() {
    use ferroui_controls::Border;
    let _application = super::support::start_catalog_application();
    let page = crate::pages::BorderPage::new();
    let root: Ref<ContentPage> = page.clone().upcast();
    let window = Window::new();
    window.set_content(Some(Control::boxed(&page.clone().upcast::<Control>())));
    window.show();
    let uri = "ferres://controlcatalog/reload-tests/InlineResources.xaml";
    let baseline = include_str!("../Pages/BorderPage.xaml");
    let target: Rc<dyn ReloadTarget> = Rc::new(crate::hot_reload::component_target(
        &root, uri.into(), baseline.into(), Vec::new(),
    ));
    let dictionary = root.resources();
    dictionary.add_value("ApplicationKey", "preserved".to_string());
    let app_value = dictionary.get(&"ApplicationKey".into()).unwrap();
    let mut session = HotReloadSession::new();
    session.register_document(uri.into(), baseline.into()).unwrap();
    session.register_target(&target);
    for width in [240, 280] {
        let update = baseline.replace(
            "  <ScrollViewer", &format!(r#"<ContentPage.Resources><x:Double x:Key="RootWidth">{width}</x:Double></ContentPage.Resources>
  <ScrollViewer"#),
        ).replacen("<Border BorderBrush=", r#"<Border Width="{StaticResource RootWidth}" BorderBrush="#, 1);
        session.submit(uri.into(), update);
        let result = session.apply_pending().unwrap();
        assert_eq!(result.status, ReloadStatus::Applied, "{result:?}");
        Dispatcher::ui_thread().run_jobs(None);
        assert!(root.resources().ptr_eq(&dictionary));
        assert!(Rc::ptr_eq(&app_value, &dictionary.get(&"ApplicationKey".into()).unwrap()));
        let border = page.get_logical_descendants().find_map(|control| control.cast::<Border>()).unwrap();
        assert_eq!(border.width(), width as f64);
    }
    session.submit(uri.into(), baseline.into());
    let result = session.apply_pending().unwrap();
    assert_eq!(result.status, ReloadStatus::Applied, "{result:?}");
    assert!(!dictionary.contains_key(&"RootWidth".into()));
    assert!(Rc::ptr_eq(&app_value, &dictionary.get(&"ApplicationKey".into()).unwrap()));
    window.close();
}

#[test]
fn generated_manifest_allows_binding_element_path_edits() {
    use ferroui_controls::{CheckBox, ContentControl, TextBlock};
    use ferroui_markup_xaml::FromXamlObject;
    let _application = super::support::start_catalog_application();
    let uri = "ferres://catalog-binding-contract/View.xaml";
    let source = r#"<TextBlock xmlns="https://github.com/ferroui"><TextBlock.Text><Binding Path="Tag"/></TextBlock.Text></TextBlock>"#;
    let mut manifest = ferroui_hot_reload::ReloadManifest::parse(
        include_str!(env!("FERROUI_HOT_RELOAD_MANIFEST")),
    ).unwrap();
    manifest.sources.clear();
    manifest.sources.insert(uri.into(), source.into());
    let mut session = manifest.session().unwrap();
    let model = CheckBox::new();
    model.set_tag(Some(Rc::new("tag value".to_string())));
    model.set_content(Some(Rc::new("content value".to_string())));
    let host = ContentControl::new();
    host.set_data_context(Some(Rc::new(model)));
    let target: Rc<dyn ReloadTarget> = Rc::new(ferroui_hot_reload::ContentReloadTarget::new(
        &host, uri.into(), Vec::new(), RuntimeXamlLoaderConfiguration::default(),
    ));
    session.register_target(&target);
    for (source, expected) in [
        (source.to_owned(), "tag value"),
        (source.replace("Path=\"Tag\"", "Path=\"Content\""), "content value"),
    ] {
        session.submit(uri.into(), source);
        let outcome = session.apply_pending().unwrap();
        assert_eq!(outcome.status, ReloadStatus::Applied, "{outcome:?}");
        Dispatcher::ui_thread().run_jobs(None);
        let text = Ref::<TextBlock>::from_xaml_object(&host.content().unwrap()).unwrap();
        assert_eq!(text.text().as_deref(), Some(expected));
    }
}

#[test]
fn checkbox_application_state_adapter_preserves_input_and_respects_new_markup() {
    use ferroui_base::controls::NameScope;
    use ferroui_controls::CheckBox;
    let _application = super::support::start_catalog_application();
    let page = crate::pages::CheckBoxPage::new();
    let root: Ref<ContentPage> = page.clone().upcast();
    let window = Window::new();
    window.set_content(Some(Control::boxed(&page.clone().upcast::<Control>())));
    window.show();
    Dispatcher::ui_thread().run_jobs(None);
    let find = || {
        NameScope::get_name_scope(&root).unwrap()
            .find("ReloadCheckBox").unwrap().cast::<CheckBox>().unwrap()
    };
    let uri = "ferres://catalog-state-tests/CheckBoxPage.xaml";
    let source = include_str!("../Pages/CheckBoxPage.xaml");
    // Exercise the actual build metadata while isolating this session's document
    // URI from the application's own development session.
    let mut manifest = ferroui_hot_reload::ReloadManifest::parse(
        include_str!(env!("FERROUI_HOT_RELOAD_MANIFEST")),
    ).unwrap();
    manifest.sources.clear();
    manifest.sources.insert(uri.into(), source.into());
    let mut session = manifest.session().unwrap();
    let target: Rc<dyn ReloadTarget> = Rc::new(crate::hot_reload::component_target(
        &root, uri.into(), source.into(), Vec::new(),
    ));
    session.register_target(&target);
    for revision in 1..=3 {
        let old = find();
        old.set_is_checked(Some(true));
        let retired = old.downgrade();
        let update = source.replace("A check box control", &format!("Check box revision {revision}"));
        session.submit(uri.into(), update.clone());
        let failed = session.apply_pending_with(|| Err(ReloadError::Failed("rollback".into()))).unwrap();
        assert_eq!(failed.status, ReloadStatus::Failed, "{failed:?}");
        Dispatcher::ui_thread().run_jobs(None);
        assert!(find().ptr_eq(&old));
        assert_eq!(old.is_checked(), Some(true));
        session.submit(uri.into(), update);
        let applied = session.apply_pending().unwrap();
        assert_eq!(applied.status, ReloadStatus::Applied, "{applied:?}");
        Dispatcher::ui_thread().run_jobs(None);
        assert!(!find().ptr_eq(&old));
        assert_eq!(find().is_checked(), Some(true));
        drop(old);
        assert!(retired.upgrade().is_none(), "custom state retained an old check box");
    }
    session.submit(uri.into(), source.replace(
        "x:Name=\"ReloadCheckBox\"", "x:Name=\"ReloadCheckBox\" IsChecked=\"False\"",
    ));
    let applied = session.apply_pending().unwrap();
    assert_eq!(applied.status, ReloadStatus::Applied, "{applied:?}");
    Dispatcher::ui_thread().run_jobs(None);
    assert_eq!(find().is_checked(), Some(false));
    find().set_is_checked(Some(true));
    session.submit(uri.into(), source.into());
    let applied = session.apply_pending().unwrap();
    assert_eq!(applied.status, ReloadStatus::Applied, "{applied:?}");
    Dispatcher::ui_thread().run_jobs(None);
    assert_eq!(find().is_checked(), Some(false), "removed authored value must not restore stale input");
    window.close();
}

#[test]
fn registered_catalog_root_binding_uses_preserved_page_and_window() {
    let _application = super::support::start_catalog_application();
    let page = crate::pages::BorderPage::new();
    let root: Ref<ContentPage> = page.clone().upcast();
    let window = Window::new();
    window.set_content(Some(Control::boxed(&page.clone().upcast::<Control>())));
    window.show();
    let uri = "ferres://controlcatalog/reload-tests/BorderPage.xaml";
    let source = include_str!("../Pages/BorderPage.xaml");
    let target: Rc<dyn ReloadTarget> = Rc::new(crate::hot_reload::component_target(
        &root, uri.into(), source.into(), Vec::new(),
    ));
    let mut session = HotReloadSession::new();
    session.register_document(uri.into(), source.into()).unwrap();
    session.register_target(&target);
    root.set_height(320.0);
    let edited = source.replacen(
        "<ContentPage ", r#"<ContentPage Width="{Binding Height, RelativeSource={RelativeSource Self}}" "#, 1,
    );
    assert_ne!(edited, source);
    let old = root.content();
    session.submit(uri.into(), edited.clone());
    let finalized = std::cell::Cell::new(false);
    let outcome = session.apply_pending_with(|| {
        finalized.set(true);
        assert_eq!(root.width(), 320.0);
        Err(ReloadError::Failed("later target failed".into()))
    }).unwrap();
    assert_eq!(outcome.status, ReloadStatus::Failed, "{outcome:?}");
    assert!(finalized.get(), "fixture failed before testing transaction rollback: {outcome:?}");
    assert_eq!(root.content(), old);
    assert!(root.width().is_nan());
    session.submit(uri.into(), edited);
    let outcome = session.apply_pending().unwrap();
    assert_eq!(outcome.status, ReloadStatus::Applied, "{outcome:?}");
    assert_eq!(root.width(), 320.0);
    root.set_height(400.0);
    assert_eq!(root.width(), 400.0);
    assert_eq!(window.content(), Some(Control::boxed(&page.clone().upcast::<Control>())));
    window.close();
}

#[test]
fn application_scope_releases_catalog_watcher_before_thread_teardown() {
    let released = {
        let _application = super::support::start_catalog_application();
        let _page = crate::pages::ListBoxPage::new();
        crate::hot_reload::session_release_check()
    };
    assert!(released(), "application scope retained the catalog reader");
}

#[test]
fn list_box_reload_preserves_shared_selection_on_commit_and_rollback() {
    let _application = super::support::start_catalog_application();
    let page = crate::pages::ListBoxPage::new();
    let context = page.data_context().unwrap();
    let model = context
        .downcast_ref::<crate::view_models::ListBoxPageViewModel>()
        .unwrap();
    let root: Ref<ContentPage> = page.clone().upcast();
    let window = Window::new();
    window.set_width(900.0);
    window.set_height(650.0);
    window.set_content(Some(Control::boxed(&page.clone().upcast::<Control>())));
    window.show();
    Dispatcher::ui_thread().run_jobs(None);
    model.selection().select(42);
    assert_eq!(model.selection().selected_index(), 42);
    // The catalog watcher owns its original URI. This transaction fixture uses
    // a separate document identity while exercising the actual page and model.
    let uri = "ferres://catalog-reload-tests/ListBoxPage.xaml".to_string();
    let source = include_str!("../Pages/ListBoxPage.xaml");
    let mut configuration = RuntimeXamlLoaderConfiguration::default();
    configuration.local_assembly = Some(&crate::register_types::ASSEMBLY);
    configuration.use_compiled_bindings_by_default = true;
    let mut session = HotReloadSession::new();
    session
        .register_document(uri.clone(), source.into())
        .unwrap();
    let target: Rc<dyn ReloadTarget> = Rc::new(ComponentReloadTarget::content_page(
        &root,
        uri.clone(),
        source.into(),
        configuration,
    ));
    session.register_target(&target);
    for index in 0..3 {
        let before = root.content();
        let retired = ferroui_base::metadata::from_markup_value::<Ref<Control>>(&before)
            .unwrap()
            .downgrade();
        let update = source.replace(
            "Hosts a collection",
            &format!("Revision {index}: Hosts a collection"),
        );
        session.submit(uri.clone(), update.clone());
        assert_eq!(
            session
                .apply_pending_with(|| Err(ReloadError::Failed("rollback fixture".into())))
                .unwrap()
                .status,
            ReloadStatus::Failed
        );
        Dispatcher::ui_thread().run_jobs(None);
        assert_eq!(root.content(), before);
        assert_eq!(
            model.selection().selected_index(),
            42,
            "rollback changed view-model selection"
        );
        session.submit(uri.clone(), update);
        assert_eq!(
            session.apply_pending().unwrap().status,
            ReloadStatus::Applied
        );
        Dispatcher::ui_thread().run_jobs(None);
        assert_ne!(root.content(), before);
        assert!(Rc::ptr_eq(&root.data_context().unwrap(), &context));
        assert_eq!(
            model.selection().selected_index(),
            42,
            "reload changed view-model selection"
        );
        drop(before);
        Dispatcher::ui_thread().run_jobs(None);
        assert!(
            retired.upgrade().is_none(),
            "retired tree retained after reload"
        );
    }
    window.close();
}

#[test]
fn application_resource_include_is_discovered_and_replaced() {
    use ferroui_controls::Application;
    let _application = super::support::start_catalog_app();
    let application = Application::current().unwrap();
    let resources = application.resources();
    let mut configuration = RuntimeXamlLoaderConfiguration::default();
    configuration.local_assembly = Some(&crate::register_types::ASSEMBLY);
    let uri = "ferres://controlcatalog/CustomThemes.xaml";
    let source = include_str!("../CustomThemes.xaml");
    let mut session = HotReloadSession::new();
    session
        .register_document(uri.into(), source.into())
        .unwrap();
    session.track_resource_includes(&resources, configuration);
    let old = resources.merged_dictionaries().to_vec();
    session.submit(uri.into(), source.replace("#2B2B33", "#2C2C34"));
    let result = session.apply_pending().unwrap();
    assert_eq!(result.status, ReloadStatus::Applied, "{result:?}");
    let current = resources.merged_dictionaries().to_vec();
    assert_eq!(current.len(), old.len());
    assert!(!Rc::ptr_eq(&current[0], &old[0]));
    assert!(old[0].owner().is_none());
    assert!(current[0].owner().is_some());
    assert!(application.resources().ptr_eq(&resources));
}
