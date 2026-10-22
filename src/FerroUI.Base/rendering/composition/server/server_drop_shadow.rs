use super::{
    impl_server_render_resource, IAnimatedServerObject, IServerObject, IServerRenderResource, ServerCompositor,
    ServerRenderResource,
};
use crate::media::{BoxShadow, BoxShadows, Color};
use crate::rendering::composition::generated::{ServerDropShadowHooks, ServerDropShadowProps};
use crate::rendering::composition::transport::BatchStreamReader;
use crate::Rect;
use std::any::{Any, TypeId};
use std::rc::{Rc, Weak};
use std::time::Duration;

/// The server-side counterpart of a
/// [`DropShadow`](crate::rendering::composition::DropShadow).
pub struct ServerDropShadow {
    base: ServerRenderResource,
    props: ServerDropShadowProps,
}

impl ServerDropShadow {
    /// Creates the shadow, active: the UI-thread shadow activates its server
    /// object when it is created.
    pub fn new(compositor: &Rc<ServerCompositor>) -> Rc<ServerDropShadow> {
        let shadow = Rc::new_cyclic(|this: &Weak<ServerDropShadow>| ServerDropShadow {
            base: ServerRenderResource::new(
                compositor,
                this.clone() as Weak<dyn IAnimatedServerObject>,
                this.clone() as Weak<dyn IServerRenderResource>,
            ),
            props: ServerDropShadowProps::new(),
        });
        shadow.base.object().activate();
        shadow
    }

    /// The properties of the shadow.
    pub fn props(&self) -> &ServerDropShadowProps {
        &self.props
    }

    pub fn is_disposed(&self) -> bool {
        self.base.is_disposed()
    }

    /// The shadow as the box shadow of a rectangle: offset, blur radius, and
    /// the colour with the opacity of the shadow in its alpha.
    ///
    /// Deviation (winui-composition.md): the shadow is the box shadow of the
    /// rectangle of the visual; `Mask` and `SourcePolicy` are stored and not
    /// used yet (phase 5), and the Z offset is ignored.
    pub fn box_shadow(&self) -> BoxShadow {
        let color = self.props.color();
        let opacity = self.props.opacity().clamp(0.0, 1.0);
        let alpha = (color.a as f32 * opacity).round().clamp(0.0, 255.0) as u8;
        let offset = self.props.offset();
        BoxShadow {
            offset_x: offset.x,
            offset_y: offset.y,
            blur: self.props.blur_radius().max(0.0) as f64,
            spread: 0.0,
            color: Color::from_argb(alpha, color.r, color.g, color.b),
            is_inset: false,
        }
    }

    /// The box shadows the shadow draws under a rectangle: empty when the
    /// shadow is invisible.
    pub fn box_shadows(&self) -> BoxShadows {
        let shadow = self.box_shadow();
        if shadow.color.a == 0 {
            return BoxShadows::default();
        }
        BoxShadows::new(shadow)
    }

    /// The area the shadow of `rect` covers, together with `rect`.
    pub fn bounds(&self, rect: Rect) -> Rect {
        self.box_shadows().transform_bounds(rect)
    }
}

impl_server_render_resource!(
    ServerDropShadow,
    base,
    composition_property: ServerDropShadowProps::get_composition_property
);

impl ServerDropShadowHooks for ServerDropShadow {}

impl IServerObject for ServerDropShadow {
    fn deserialize_changes_core(&self, reader: &mut BatchStreamReader<'_>, committed_at: Duration) {
        self.props.deserialize_changes_core(self, reader, committed_at);
    }

    fn values_invalidated(&self) {
        self.base.core().invalidated(self);
    }

    fn dispose(&self) {
        self.base.core().dispose();
    }

    fn as_render_resource(self: Rc<Self>) -> Option<Rc<dyn IServerRenderResource>> {
        Some(self)
    }

    fn as_animated(self: Rc<Self>) -> Option<Rc<dyn IAnimatedServerObject>> {
        Some(self)
    }

    fn get_props(&self, type_id: TypeId) -> Option<&dyn Any> {
        self.props.find_props(type_id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }
}
