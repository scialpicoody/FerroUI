//! Reload integration with application-owned view models and shared selection.
use ferroui_base::{threading::Dispatcher, Ref};
use ferroui_controls::{selection::ISelectionModel, ContentPage, Control, Window};
use ferroui_hot_reload::{
    ComponentReloadTarget, HotReloadSession, ReloadError, ReloadStatus, ReloadTarget,
};
use ferroui_markup_xaml::RuntimeXamlLoaderConfiguration;
use std::rc::Rc;

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
