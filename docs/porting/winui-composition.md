# The WinUI 3 composition API

This document is the inventory and the plan of the port of the WinUI 3 composition API
(`Microsoft.UI.Composition`) and of the XAML pieces that sit on it in microsoft-ui-xaml
(`ConnectedAnimationService`, `ConnectedAnimation`, `ElementCompositionPreview`, the `Translation`
facade of `UIElement`) to FerroUI. It records what FerroUI already has, what the port adds, in which
order, and every place where the result differs from WinUI.

## 1. Sources

- **The compositor of FerroUI.** `src/FerroUI.Base/rendering/composition` is the port of the
  compositor of the upstream framework: `Compositor`, `CompositionVisual` and its container,
  solid colour, surface, draw list and custom visuals, `CompositionPropertySet`,
  `ExpressionAnimation`, the key frame animations of the schema, implicit animations, animation
  groups, the expression language, the composition brushes, the server/transport split and the
  drawing surfaces. Its object model is generated from `scripts/composition-schema.xml` by
  `scripts/generate_composition.py`. The port builds on it and does not duplicate any of it.
- **The connected animations of the upstream framework.** `src/FerroUI.Controls/animation` is the
  port of the upstream `ConnectedAnimationService`, which flies a control over the overlay layer
  with UI-thread animations. The WinUI port replaces its engine with the composition engine of
  WinUI and keeps its public members (section 5.1).
- **microsoft-ui-xaml**, branch `winui3/main`, commit `7e4b0c2096be402f51cc8f7d203ebf67cff183d0`
  (local checkout `/Users/wieslawsoltes/GitHub/microsoft-ui-xaml`, read-only, MIT). The XAML-side
  code is ported from it:
  - `dxaml/xcp/components/animation/ConnectedAnimation.cpp`, `ConnectedAnimationService.cpp` and
    their headers: the state machine, the rectangles, the snapshot brushes and the sprite
    animations of a connected animation;
  - `dxaml/xcp/dxaml/lib/ConnectedAnimation*_partial.cpp`, `GravityConnectedAnimationConfiguration_partial.cpp`,
    `DirectConnectedAnimationConfiguration_partial.cpp`, `BasicConnectedAnimationConfiguration_partial.h`:
    the public classes and the three configurations;
  - `dxaml/xcp/core/core/elements/ConnectedAnimationRoot.cpp`: the layer the animations fly in;
  - `dxaml/xcp/dxaml/lib/ElementCompositionPreview.cpp`: the `ElementCompositionPreview` statics.
- **`Microsoft.UI.Composition`** is closed source (it ships in the Windows App SDK). It is ported
  from its public API reference (learn.microsoft.com, the `Microsoft.UI.Composition` namespace of
  the Windows App SDK 1.6) and from how microsoft-ui-xaml uses it (`DCompTreeHost`, `HWCompNode`,
  the connected animation code). Where the reference does not say how something behaves, the port
  says what it chose in the deviations (section 7).

The upstream framework name never appears in `src`, `samples` or `tests`; this page and the
`scripts/` folder may name it.

## 2. How the WinUI types map onto FerroUI

| WinUI | FerroUI |
|---|---|
| `Microsoft.UI.Composition` | `ferroui_base::rendering::composition` (the compositor and its objects) |
| `Microsoft.UI.Xaml.Hosting.ElementCompositionPreview` | `ferroui_controls::hosting::ElementCompositionPreview` (it reaches `ScrollViewer`, a control); the visual-tree half stays `ferroui_base::rendering::composition::ElementComposition` |
| `Microsoft.UI.Xaml.Media.Animation.ConnectedAnimation*` | `ferroui_controls::animation` |
| `UIElement` | `Visual` (the element that has a composition visual) |
| handoff visual (`GetElementVisual`) | the `CompositionDrawListVisual` the compositing renderer creates for a visual |
| hand-in visual (`SetElementChildVisual`) | the child composition visual of a visual, shown after its children |
| `ICompositionEasingFunction` | `ferroui_base::animation::easings::Easing` (`Rc<dyn IEasing>` inside); `CompositionEasingFunction` has the WinUI factories |
| `Vector2`, `Vector3`, `Vector4`, `Quaternion`, `Matrix3x2`, `Matrix4x4` | `ferroui_base::numerics` (the expression language types); visual properties keep the types of the compositor (`Vector`, `Vector3D`, `Matrix`) |
| `TimeSpan` | `std::time::Duration` |
| events (`Completed`) | handler lists with a disposable registration, as elsewhere in FerroUI |

Property names used by `StartAnimation` and by expressions are the WinUI names (`Offset`,
`Opacity`, `Scale`, `Translation`, `Offset.X`, ...), so an expression or a target written for
WinUI works unchanged.

## 3. Inventory

Status: **has** (present with the WinUI members and behaviour), **partial** (present, some
members or behaviour missing, said in the notes), **missing** (not in FerroUI before this port).
The last column is the phase of section 4 that brings the type to **has**, or the decision when
it does not.

### 3.1 Objects, compositor, property sets

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `CompositionObject.Compositor`, `Dispose`, `ImplicitAnimations` | has | `CompositionObject` | - |
| `CompositionObject.StartAnimation`, `StopAnimation`, `StartAnimationGroup`, `StopAnimationGroup` | has | Sub-channel targets of vector properties (`Offset.X`, `Scale.XY`, components X, Y, Z, W in any case) animate those components only; sub-channels of one property run together, an animation of the whole property replaces them. `StopAnimation` applies the stop behavior. | 1 |
| `CompositionObject.Properties` | partial | `CompositionObject::properties()`: a live property set of the object, created on first use. An expression reads its entries through a reference to the set (`set_reference_parameter("props", obj.properties())`, then `props.Name`), not as members of the object (`obj.Name`). | 3 |
| `CompositionObject.Comment` | has | `comment`, `set_comment`; empty by default. | 3 |
| `CompositionObject.TryGetAnimationController`, `StartAnimation(name, animation, controller)`, `Compositor.CreateAnimationController`, `AnimationController` (`Pause`, `Resume`, `Progress`, `PlaybackRate`, `ProgressBehavior`, `MaxPlaybackRate`, `MinPlaybackRate`), `AnimationControllerProgressBehavior` | partial | `try_get_animation_controller`, `start_animation_with_controller`, `create_animation_controller`. The controller is a live property set (`Progress`, `PlaybackRate` animatable, the controller referenceable from expressions) that the server instance reads every frame: a paused animation stands still and does not end, a rate scales (a negative one reverses) its time, a new `Progress` moves it there (with or without the delay). Key frame and spring animations can be controlled; expression animations ignore a controller. `Progress` reads the value last assigned, not the position of the running animation; `TryGetAnimationController` gives a controller for any animation started on the property through the object and not stopped, also one that ended by itself. | 3 |
| `CompositionPropertySet` `Insert*`, `TryGet*` | has | Scalar, vector, matrix, quaternion, colour, boolean. | - |
| `Compositor.Create*` for the types below | partial | The factories of the existing types exist; the new types bring theirs. | 1-6 |
| `Compositor.CreatePropertySet` | has | `create_property_set`. | 1 |
| `Compositor.CreateScopedBatch`, `GetCommitBatch`, `CompositionScopedBatch`, `CompositionCommitBatch` | has | `create_scoped_batch`, `get_commit_batch`, `CompositionBatchTypes`. Key frame animations started while a batch is active take a completion guard that the server releases when the animation ends, is stopped or replaced, or its target is disposed; `Completed` is raised on the thread of the compositor. Expression animations belong to no batch. | 1 |
| `Compositor.RequestCommitAsync` | has | `request_commit_async`. | - |
| `Compositor.GlobalPlaybackRate`, `MinGlobalPlaybackRate`, `MaxGlobalPlaybackRate` | has | `global_playback_rate`, `set_global_playback_rate` (0 to 1000, applied from the next frame), `MIN_GLOBAL_PLAYBACK_RATE`, `MAX_GLOBAL_PLAYBACK_RATE`. The server scales the clock the animations start and are evaluated on. | 3 |

### 3.2 Visuals

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `Visual.Offset`, `Size`, `Opacity`, `IsVisible`, `AnchorPoint`, `CenterPoint`, `RotationAngle`, `Orientation`, `Scale`, `Parent` | has | `CompositionVisual`; `IsVisible` is `visible`. | - |
| `Visual.Translation` (property set entry in WinUI) | has | A real animatable property of every visual. | - |
| `Visual.TransformMatrix` | partial | A 2D `Matrix`, not a `Matrix4x4`; the 3D part is ignored by a 2D renderer. | deviation |
| `Visual.Clip` (`CompositionClip`) | has | `composition_clip` / `set_composition_clip` on every visual (`clip` is the geometry clip of the upstream compositor); applied by the render walk in the space of the visual, after its transform, and part of the own clip of the visual (bounds, dirty rects). | 1 |
| `Visual.RotationAngleInDegrees`, `RotationAxis` | missing | Degrees: a view of `RotationAngle`. Axis: only the Z axis rotates in 2D. | 2 |
| `Visual.RelativeOffsetAdjustment`, `RelativeSizeAdjustment` | missing | Offset and size relative to the size of the parent. | 2 |
| `Visual.IsHitTestVisible`, `IsPixelSnappingEnabled`, `BorderMode`, `BackfaceVisibility`, `CompositeMode`, `ParentForTransform` | missing | `ParentForTransform` is close to the existing adorned visual. Backface and 3D modes have no meaning in a 2D renderer. | 2 / deviation |
| `ContainerVisual.Children` (`InsertAbove`, `InsertBelow`, `InsertAtTop`, `InsertAtBottom`, `Remove`, `RemoveAll`, `Count`) | has | `CompositionVisualCollection`. | - |
| `SpriteVisual` (`Brush`, `Shadow`) | has | `SpriteVisual`, `create_sprite_visual`. The shadow of its rectangle, then the brush over `(0, 0, size)`, then the children; brush and shadow are observed render resources. Does not clip to its bounds when created, as a WinUI visual. | 1 |
| `ShapeVisual` (`Shapes`, `ViewBox`) | missing | | 2 |
| `LayerVisual` (`Effect`, `Shadow`) | missing | Children composed into a layer, then the effect or shadow. | 2 |
| `RedirectVisual` (`Source`) | missing | Shows another visual's subtree. | 2 |

### 3.3 Brushes and surfaces

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `CompositionColorBrush` (`Color`) | has | `CompositionSolidColorBrush`; `create_color_brush` and `create_color_brush_with_color` are the WinUI-named factories. | - |
| `CompositionGradientBrush` (`ColorStops`, `ExtendMode`) | partial | Stops and spread method exist. Missing: `MappingMode`, `InterpolationSpace`, the brush transform (`AnchorPoint`, `CenterPoint`, `Offset`, `RotationAngle`, `Scale`, `TransformMatrix`). | 2 |
| `CompositionLinearGradientBrush` (`StartPoint`, `EndPoint`) | partial | As relative points, not `Vector2` with a mapping mode. | 2 |
| `CompositionRadialGradientBrush` (`EllipseCenter`, `EllipseRadius`, `GradientOriginOffset`) | partial | `Center`, `RadiusX/Y`, `GradientOrigin` of the upstream brush. | 2 |
| `CompositionSurfaceBrush` (`Surface`, `Stretch`, alignment ratios, `BitmapInterpolationMode`, transform, `SnapToPixels`) | has | `CompositionSurfaceBrush`, `create_surface_brush`, `create_surface_brush_with_surface`. Painted by the visual that shows it (`paint_composition_brush` on the server), not a brush of the drawing context. `CompositionStretch` gains `Uniform` and `UniformToFill`. The interpolation mode maps to the one quality setting of the drawing context. | 1 |
| `CompositionNineGridBrush` | missing | | 2 |
| `CompositionMaskBrush` (`Source`, `Mask`) | missing | An opacity mask of the drawing context. | 2 |
| `CompositionBackdropBrush`, `CompositionHostBackdropBrush` | missing | Needs the pixels behind the visual. | 6 |
| `CompositionEffectBrush`, `CompositionEffectFactory`, `CompositionEffectSourceParameter` | missing | The Win2D effect graph. | 6 (scoped) |
| `ICompositionSurface` | has | `ICompositionSurface` (UI thread), implemented by `CompositionSurface` (and so the drawing surface), `CompositionVisualSurface` and `LoadedImageSurface`; on the server `IServerCompositionSurface` gains `natural_size` and `draw`, which the surface visual uses too. | 1 |
| `CompositionVisualSurface` (`SourceVisual`, `SourceOffset`, `SourceSize`) | has | `create_visual_surface`. Live: the source subtree is rendered where the surface is drawn, ignoring the visibility, opacity, transform and clip of the source itself; changes of the subtree are followed while the source is attached to a target. A surface that shows an ancestor of its sprite is not drawn inside itself. `freeze()` (not public in WinUI) renders the source once into a layer. | 1 |
| `CompositionDrawingSurface` | partial | The upstream class takes imported GPU images; WinUI draws with Direct2D interop, which has no counterpart. | deviation |
| `LoadedImageSurface` (XAML) | partial | `LoadedImageSurface::from_bitmap` and `start_load_from_stream` (decoded at once, `load_status` instead of `LoadCompleted`); `natural_size`, `decoded_size`, `decoded_physical_size`. No URI loading and no decode size. | 1 |

### 3.4 Shapes and geometry

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `CompositionShape` (`CenterPoint`, `Offset`, `RotationAngle`, `RotationAngleInDegrees`, `Scale`, `TransformMatrix`) | missing | | 2 |
| `CompositionSpriteShape` (`Geometry`, `FillBrush`, `StrokeBrush`, `StrokeThickness`, dash array, caps, join, miter limit, `IsStrokeNonScaling`) | missing | | 2 |
| `CompositionContainerShape` (`Shapes`) | missing | | 2 |
| `CompositionGeometry` (`TrimStart`, `TrimEnd`, `TrimOffset`) | missing | | 2 |
| `CompositionRectangleGeometry`, `CompositionRoundedRectangleGeometry`, `CompositionEllipseGeometry`, `CompositionLineGeometry`, `CompositionPathGeometry` + `CompositionPath` | missing | | 2 |
| `CompositionViewBox` | missing | | 2 |

### 3.5 Clips

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `CompositionClip` (`AnchorPoint`, `CenterPoint`, `Offset`, `RotationAngle`, `RotationAngleInDegrees`, `Scale`, `TransformMatrix`) | has | `CompositionClip`; the transform is animatable; `AnchorPoint` is relative to the size of the visual; `RotationAngleInDegrees` is a view of `RotationAngle` (not an animation target). | 1 |
| `InsetClip` (`LeftInset`, `TopInset`, `RightInset`, `BottomInset`) | has | `create_inset_clip`, `create_inset_clip_with_insets`; the insets are animatable and their animations re-render the visual. | 1 |
| `RectangleClip` (edges and corner radii) | has | `create_rectangle_clip`, `create_rectangle_clip_with`, `create_rectangle_clip_with_radii`; a rounded clip pushes a rounded rectangle. | 1 |
| `GeometricClip` (`Geometry`, `ViewBox`) | missing | Needs the geometries of phase 2. | 2 |

### 3.6 Lights and shadows

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `DropShadow` (`BlurRadius`, `Color`, `Offset`, `Opacity`, `Mask`, `SourcePolicy`) | partial | `create_drop_shadow`; on sprite visuals, the box shadow of the rectangle of the visual (the bounds of the visual include it); `BlurRadius`, `Color`, `Offset`, `Opacity` animatable. `Mask` and `SourcePolicy` are stored and not used yet. | 1 (rectangular), 5 (masks) |
| `CompositionLight` (`Targets`, `ExclusionsFromTargets`, `IsEnabled`) | missing | | 5 |
| `AmbientLight`, `DistantLight`, `PointLight`, `SpotLight` | missing | Diffuse lighting of flat visuals. | 5 |

### 3.7 Animations and easing

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `ScalarKeyFrameAnimation`, `Vector2/3/4KeyFrameAnimation`, `ColorKeyFrameAnimation`, `QuaternionKeyFrameAnimation`, `BooleanKeyFrameAnimation` | has | From the schema. | - |
| `KeyFrameAnimation` `DelayBehavior`, `DelayTime`, `Direction`, `Duration`, `IterationBehavior`, `IterationCount`, `InsertExpressionKeyFrame` | has | | - |
| `KeyFrameAnimation.StopBehavior` | has | Applied by `StopAnimation` (upstream stores it and does not apply it); an animation that is replaced leaves the current value. | 1 |
| `KeyFrameAnimation.KeyFrameCount` | has | `key_frame_count`. | 1 |
| `ColorKeyFrameAnimation.InterpolationColorSpace` (`CompositionColorSpace`) | has | `interpolation_color_space`: `Auto` and `Rgb` interpolate the stored channels, `Hsl` hue (the short way), saturation and lightness, `RgbLinear` the channels in linear light, `HslLinear` HSL of the linear-light colour. `Auto` is RGB. | 3 |
| `PathKeyFrameAnimation` | missing | Needs `CompositionPath` (phase 2). | 3 |
| `ExpressionAnimation` (`Expression`) | has | | - |
| `CompositionAnimation.Set*Parameter`, `SetReferenceParameter`, `ClearParameter`, `ClearAllParameters`, `Target` | has | | - |
| `CompositionAnimation.SetBooleanParameter`, `SetExpressionReferenceParameter` | has | `set_boolean_parameter`; `set_expression_reference_parameter` takes a composition object (`Rc<dyn AsCompositionObject>`) and is resolved as `set_reference_parameter` (WinUI's `IAnimationObject` is not ported, see the deviations). | 3 |
| `CompositionAnimation.InitialValueExpressions` (`InitialValueExpressionCollection`) | has | `initial_value_expressions()`: the map (`insert`, `lookup`, `has_key`, `remove`, `clear`, `size`, `get_view`). Parsed when the animation starts (an expression that cannot be parsed fails the start); on the server, before the instance is initialized, each expression is evaluated against the target and the parameters (`this.StartingValue` = the property's value, the animated property read as it was before the start) and the named property of the target is set; the animated property starts from its initial value. Key frame, expression and spring animations. | 3 |
| `CompositionAnimationGroup` | has | | - |
| `ImplicitAnimationCollection` | has | | - |
| `NaturalMotionAnimation` (`DelayBehavior`, `DelayTime`, `StopBehavior`), `ScalarNaturalMotionAnimation`, `Vector2NaturalMotionAnimation`, `Vector3NaturalMotionAnimation` (`FinalValue`, `InitialValue`, `InitialVelocity`), `SpringScalarNaturalMotionAnimation`, `SpringVector2NaturalMotionAnimation`, `SpringVector3NaturalMotionAnimation` (`DampingRatio`, `Period`), `Compositor.CreateSpring*Animation` | has | `create_spring_scalar_animation`, `create_spring_vector2_animation`, `create_spring_vector3_animation`. Each component moves on a damped harmonic oscillator (closed form, `spring_state`) from `InitialValue` (or the current value) with `InitialVelocity` to `FinalValue` (or the new value of an implicit start, or the current value); the animation ends when every component is within 1e-4 of the size of the motion and nearly at rest, sets the final value and releases its batches. Defaults chosen: `DampingRatio` 1, `Period` 50 ms. A spring with damping 0 is an infinite animation. Works on visual properties (a `Vector3` spring on `Offset` evaluates to `Vector3D`) and property set entries, explicit or implicit. The `Bounce*NaturalMotionAnimation` classes of newer SDKs are not ported. | 3 |
| `CompositionScopedBatch`, `CompositionCommitBatch` | has | See 3.1. | 1 |
| `CubicBezierEasingFunction` (`ControlPoint1`, `ControlPoint2`) | has | The curve of `SplineEasing`. The easing functions are `IEasing` implementations, not composition objects; `CompositionEasingFunction` has the factories of WinUI 1.4+ and `Compositor` the `Create*EasingFunction` factories. | 1 |
| `LinearEasingFunction` | has | `LinearEasingFunction` (the curve of `LinearEasing`). | 1 |
| `StepEasingFunction` (`StepCount`, `InitialStep`, `FinalStep`, `IsInitialStepSingleFrame`, `IsFinalStepSingleFrame`) | has | The steps share the duration; a single-frame step is shown only at the start or the end. | 1 |
| `BackEasingFunction` (`Mode`, `Amplitude`), `BounceEasingFunction` (`Mode`, `Bounces`, `Bounciness`), `CircleEasingFunction` (`Mode`), `ElasticEasingFunction` (`Mode`, `Oscillations`, `Springiness`), `ExponentialEasingFunction` (`Mode`, `Exponent`), `PowerEasingFunction` (`Mode`, `Power`), `SineEasingFunction` (`Mode`) | has | The parametric curves of the XAML easing functions, with `CompositionEasingFunctionMode`; read-only members as in WinUI. | 1 |

### 3.8 Expression language

| Feature | Status | Notes | Phase |
|---|---|---|---|
| Operators (`+ - * / %`, comparisons, `&& || !`, `?:`), literals, member access, swizzles (`.XY`), `this.StartingValue`, `this.CurrentValue`, `this.FinalValue`, `this.Target` | has | `expressions/` | - |
| Functions `Abs`, `ACos`, `ASin`, `ATan`, `Ceil`, `Clamp`, `ColorLerp`, `ColorLerpRGB`, `ColorRGB`, `Concatenate`, `Cos`, `Distance`, `DistanceSquared`, `Floor`, `Inverse`, `Length`, `LengthSquared`, `Lerp`, `Ln`, `Log10`, `Matrix3x2`, `Matrix4x4`, `Max`, `Min`, `Mod`, `Normalize`, `Pow`, `Quaternion`, `Round`, `Scale`, `Sin`, `Slerp`, `SmoothStep`, `Sqrt`, `Square`, `Tan`, `ToDegrees`, `ToRadians`, `Transform`, `Vector2`, `Vector3`, `Vector4` | has | Every overload of the reference ("Expression Functions per Type" of `ExpressionAnimation`), plus those the WinUI expression engine accepts without listing them (the expression builder of the Windows Community Toolkit emits them): scalar `Length`, `LengthSquared`, `Distance`, `DistanceSquared`, `Scale`; per-component `Mod` of vectors; `Transform(Vector4, Matrix4x4)`. Also `Lerp` of `Matrix3x2` and `Matrix4x4` (not in the reference). `Matrix3x2(...)` and `Matrix4x4(...)` take each component from its own argument (the upstream implementation repeated the fifth). | 3 |
| Function names | has | Case-insensitive (`lerp`, `LERP`, `Asin` and `ASin` are the same function), as in WinUI, whose own expression builder writes them in lower case; also the builder's names `ColorHslLerp` and `ColorRgbLerp`. Keywords (`this.Target`, `Pi`, `True`) were already case-insensitive. | 3 |
| `ColorHsl`, `ColorLerpHSL` | has | `ColorHsl(h, s, l)`: hue in radians (0 to 2 pi, wrapping), saturation and lightness 0..1 (clamped), opaque. `ColorLerpHSL(c1, c2, p)`: `c1` at 0, `c2` at 1 as `ColorLerp`; saturation, lightness, alpha linear, hue the short way round, an achromatic colour takes the other's hue. | 3 |
| `Matrix3x2.CreateFromScale`, `CreateFromTranslation`, `CreateRotation`, `CreateScale`, `CreateSkew`, `CreateTranslation`; `Matrix4x4.CreateFromAxisAngle`, `CreateFromScale`, `CreateFromTranslation`, `CreateScale`, `CreateTranslation`; `Quaternion.CreateFromAxisAngle` | has | The static factories of the reference, with its formulas; a dotted name is parsed as one function name. `CreateTranslation` is `CreateFromTranslation`, as the reference says (upstream created a scale). Also the `System.Numerics` overloads: `Matrix3x2.CreateScale(s)`, `(x, y)`, `(scale, center)`, `CreateTranslation(x, y)`, `CreateRotation(angle, center)`, `CreateSkew(x, y)`; `Matrix4x4.CreateScale(s)`, `(x, y, z)`, `(scale, center)`, `CreateTranslation(x, y, z)`, `CreateRotationX/Y/Z(angle)` and `(angle, center)` (not in the reference). `Matrix4x4.CreateFromQuaternion` and the yaw-pitch-roll factories are not in the reference and not added. | 3 |
| Member, swizzle and parameter names | partial | Case-sensitive here (`Offset.X`, `.XY`, parameter keys). Whether WinUI matches them ignoring case is not documented. | - |
| Reference to an `InteractionTracker`, a visual's property set (`visual.Translation` after `SetIsTranslationEnabled`), a pointer property set | partial | Visuals and property sets can be referenced; the tracker comes with phase 4. | 4 |

### 3.9 Interaction

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `InteractionTracker` (position, scale, velocities, bounds, resting values, decay rates, `TryUpdate*`, states, owner callbacks) | partial | `interactions::InteractionTracker`: `Create`, `CreateWithOwner`, every value property, `InteractionSources`, `TryUpdatePosition` (with `InteractionTrackerClampingOption`), `TryUpdatePositionBy`, `TryUpdatePositionWithAnimation`, `TryUpdatePositionWithAdditionalVelocity`, `TryUpdateScale`, `TryUpdateScaleWithAnimation`, `TryUpdateScaleWithAdditionalVelocity`, request ids, the four states with a rubber band past the boundaries while interacting. Runs on the UI thread with the animation frames of the media context and publishes its values into its property set (`properties`); a reference to the tracker in an expression is a reference to that set. Custom animations are evaluated on the UI thread. Missing: tracker binding (`SetBindingMode`, `IsFromBinding` is always `false`), `TryUpdatePosition` with `InteractionTrackerPositionUpdateOption`, `AdjustPositionXIfGreaterThanThreshold`/`Y`. | 4 |
| `IInteractionTrackerOwner` | has | A trait with no-op defaults; callbacks are queued and delivered after the change, never nested. | 4 |
| `InteractionTrackerInertiaRestingValue`, `InteractionTrackerInertiaMotion`, `InteractionTrackerInertiaNaturalMotion`, `Configure*InertiaModifiers` | partial | Resting value and motion modifiers for X, Y and scale, conditions evaluated with `this.Target` as the tracker. Missing: `InteractionTrackerInertiaNaturalMotion` and `ConfigureVector2PositionInertiaModifiers` (they need the natural motion animations of phase 3). | 4 |
| `VisualInteractionSource` (`TryRedirectForManipulation`, source modes, rails, chaining, wheel configuration) | partial | `interactions::VisualInteractionSource`: `Create` (plus `create_for_element`), source modes, rails, `ManipulationRedirectionMode` and `PointerWheelConfig` (mouse wheel, Ctrl + wheel scales), the input values (`DeltaPosition`, `Position`, ...). `TryRedirectForManipulation` takes the pressed event args (or a pointer point and the element) and follows the routed pointer events of the element. Chaining modes are stored only; pinch input, touchpad redirection and expression references to a source are missing. | 4 |

### 3.10 XAML pieces

| Type / member | Status | Notes | Phase |
|---|---|---|---|
| `ConnectedAnimationService.GetForCurrentView` | partial | The upstream service has `get_for_top_level`; `get_for_current_view` is added (the service of the thread's active top level). | 1 |
| `ConnectedAnimationService.DefaultDuration`, `DefaultEasingFunction` | partial | Present; the easing default becomes WinUI's cubic Bezier (0.8, 0), (0.2, 1). | 1 |
| `ConnectedAnimationService.PrepareToAnimate`, `GetAnimation` | has | Upstream members; the engine behind them changes. | 1 |
| `ConnectedAnimation.TryStart(destination)`, `TryStart(destination, coordinatedElements)` | partial | Upstream: a control flown over the overlay layer by UI-thread animations, coordinated elements faded in. WinUI: snapshot brushes on sprite visuals animated by the compositor, coordinated elements entering from where they would be relative to the source. | 1 |
| `ConnectedAnimation.Cancel`, `IsScaleAnimationEnabled`, `SetAnimationComponent`, `Configuration`, `Completed` | partial | `Configuration`, `Completed` exist; the others are added. | 1 |
| `ConnectedAnimationConfiguration`, `GravityConnectedAnimationConfiguration` (`IsShadowEnabled`), `DirectConnectedAnimationConfiguration`, `BasicConnectedAnimationConfiguration` | partial | The classes exist; the WinUI behaviour of each (the gravity dip and shadow, the 200 ms decelerating direct animation, the basic animation without effect) is ported. | 1 |
| `ConnectedAnimationComponent` (`OffsetX`, `OffsetY`, `CrossFade`, `Scale`) | missing | | 1 |
| `ElementCompositionPreview.GetElementVisual`, `GetElementChildVisual`, `SetElementChildVisual` | has | `ElementComposition`; the WinUI-named facade is added. | 1 |
| `ElementCompositionPreview.SetImplicitShowAnimation`, `SetImplicitHideAnimation` | missing | | 1 |
| `ElementCompositionPreview.SetIsTranslationEnabled` | missing | | 1 |
| `ElementCompositionPreview.GetScrollViewerManipulationPropertySet` | missing | | 1 |
| `ElementCompositionPreview.GetPointerPositionPropertySet` | missing | | 1 |
| `UIElement.Translation`, `TranslationTransition` (`Vector3Transition`) | has | `Visual::translation` (a `Vector3D`) writes the `Translation` of the element's composition visual; `translation_transition` animates it (see the transitions row). | 1 (`Translation`), 3 (transitions) |
| `UIElement.Rotation`, `RotationAxis`, `Scale`, `CenterPoint`, `TransformMatrix` | partial | Plain properties of `Visual` (`rotation`, `rotation_axis`, `scale`, `center_point`, `transform_matrix`, with the `numerics` types), written to the element's composition visual whenever the element is rendered again; a facade never set leaves its composition property to the application. `Rotation` around the Z axis is `RotationAngle` (radians), around another axis `Orientation` (a quaternion), of which the 2D renderers show only the projection; a zero axis is the Z axis. `TransformMatrix`: its 2D affine part (`M11`, `M12`, `M21`, `M22`, `M41`, `M42`) follows the render transform in the composition `TransformMatrix`, which the composition visual applies before `Scale`, the rotation and `Translation` (WinUI applies it after the other facades). | 3 |
| `UIElement.OpacityTransition`, `RotationTransition`, `ScaleTransition`; `ScalarTransition` (`Duration`), `Vector3Transition` (`Duration`, `Components`), `Vector3TransitionComponents` | has | `ferroui_base::{ScalarTransition, Vector3Transition, Vector3TransitionComponents}` (values, copied when assigned; 300 ms and all components by default) and `Visual::set_opacity_transition`, `set_rotation_transition`, `set_scale_transition`, `set_translation_transition`. An implicit animation collection on the element's composition visual (`Opacity`, `RotationAngle` and `Orientation`, `Scale`, `Translation`): a key frame animation to `this.FinalValue` over the duration with the compositor's default easing. A `Vector3Transition` of some components starts with a key frame at 0 that holds the animated components at `this.StartingValue` and sets the others to `this.FinalValue`, so those jump at once; one of no component installs nothing. The collection is installed after the first synchronization of a new composition visual (the initial values do not animate) and before the properties are written afterwards, and replaces implicit animations the application set on the element's visual. | 3 |

## 4. Plan

The phases are ordered by value to an application: what a page of an application uses most
(connected animations between a list and a detail view, the element APIs) comes first, then the
visual building blocks, then the animation and easing breadth, then the rarer pieces. Each phase is
implemented with unit tests next to `compositor_tests.rs` and `render_thread_tests.rs` (or in the
test file of its folder), lands in the pull request of the port, and updates the inventory above.

### Phase 1: connected animations and the element APIs

What a connected animation needs, and the `ElementCompositionPreview` statics.

1. **Completion of animations**: `CompositionScopedBatch` and `CompositionCommitBatch`. An
   animation started while batches are active takes a completion guard of each; the server
   releases the guard when a key frame animation ends, is stopped or is replaced, and the
   batch raises `Completed` on the UI thread once it is ended and all its guards are released.
   `StopBehavior` is applied when an animation is stopped. Sub-channel targets (`Offset.X`,
   `Scale.XY`) for `StartAnimation`.
2. **Easing**: `CompositionEasingFunction` with the WinUI factories (cubic Bezier, linear, step,
   back, bounce, circle, elastic, exponential, power, sine, each with its mode and parameters),
   producing the `Easing` objects the key frames take.
3. **Sprite visual and surfaces**: `SpriteVisual` (`Brush`, `Shadow`), `CompositionSurfaceBrush`,
   `CompositionVisualSurface` (live), `LoadedImageSurface` (from a bitmap), `DropShadow` for
   rectangular content, the clips (`CompositionClip`, `InsetClip`, `RectangleClip`) on every
   visual.
4. **`ElementCompositionPreview`**: the WinUI facade over `ElementComposition`, the implicit
   show and hide animations (also when the element leaves the tree), `SetIsTranslationEnabled`,
   the scroll viewer manipulation and pointer position property sets; the `Translation`
   property of a visual.
5. **Connected animations**: the WinUI engine (states, rectangles, snapshot and live brushes on a
   sprite visual in a connected animation layer at the top of the window, the offset, scale,
   clip and cross-fade animations, the gravity, direct and basic configurations,
   `IsScaleAnimationEnabled`, `SetAnimationComponent`, coordinated elements, the timeout,
   `Cancel`, `Completed`) behind the upstream service's API.
6. **Catalog**: a "Composition (WinUI)" page with a list and a detail view joined by a connected
   animation, implicit show and hide animations, a pointer-driven light-like effect from the
   pointer position property set and a parallax header from the scroll viewer property set.
   Verified on macOS with the Skia and Vello backends.

### Phase 2: visuals, brushes and shapes

`ShapeVisual`, `CompositionSpriteShape`, `CompositionContainerShape`, the geometries with trimming,
`CompositionViewBox`, `GeometricClip`; `LayerVisual`, `RedirectVisual`; `CompositionNineGridBrush`,
`CompositionMaskBrush`; the WinUI members of the gradient brushes (mapping mode, `Vector2` points,
ellipse centre and radius, the brush transform); the remaining `Visual` members
(`RotationAngleInDegrees`, `RotationAxis` around Z, the relative adjustments, `IsHitTestVisible`,
`IsPixelSnappingEnabled`, `ParentForTransform`).

### Phase 3: animation and expression breadth

Spring natural motion animations; `AnimationController` and the global playback rate;
`CompositionObject.Properties` and `Comment`; `SetBooleanParameter`, `SetExpressionReferenceParameter`,
`InitialValueExpressions`; `ColorKeyFrameAnimation.InterpolationColorSpace`;
`PathKeyFrameAnimation`; the missing expression functions (`ColorHsl`, `ColorLerpHSL`, the
`Matrix3x2`, `Matrix4x4` and `Quaternion` factories); the `UIElement` facades and their
transitions (`Rotation`, `Scale`, `CenterPoint`, `TransformMatrix`, `ScalarTransition`,
`Vector3Transition`).

### Phase 4: interaction

`InteractionTracker` with its owner callbacks, states, bounds, inertia (decay rates, resting value,
motion and natural motion modifiers) and its `TryUpdate*` requests, exposed to expressions as a
referenceable object (`tracker.Position`, `tracker.Scale`); `VisualInteractionSource` fed by the
pointer and wheel input of the element whose visual it is created for, after
`TryRedirectForManipulation`.

### Phase 5: shadows and lights

`DropShadow` with a mask and the `SourcePolicy`, shadows of shape and layer visuals; the lights
(`AmbientLight`, `DistantLight`, `PointLight`, `SpotLight`) with targets and exclusions, as diffuse
lighting of flat visuals.

### Phase 6: effects (scoped)

`CompositionEffectBrush` and `CompositionEffectFactory` for the effects the drawing contexts of
FerroUI can express (Gaussian blur, opacity, cross-fade, a drop shadow), and
`CompositionBackdropBrush` on the backdrop support of the acrylic visual. The full Win2D effect
graph (the dozens of `Microsoft.Graphics.Canvas.Effects` classes and arbitrary graphs of them) is
out of scope: FerroUI has no Win2D, its drawing context has a small set of image effects, and a
general effect graph needs a shader pipeline per backend (Skia and Vello). An unsupported effect
makes `CreateEffectFactory` report `CompositionEffectFactoryLoadStatus::EffectTypeNotSupported`, as
WinUI does for an effect it cannot load.

## 5. Design notes

### 5.1 Connected animations

The upstream service and its public members stay: `ConnectedAnimationService::get_for_top_level`,
`prepare_to_animate`, `get_animation`, `default_duration`, `default_easing_function`;
`ConnectedAnimation::try_start`, `try_start_with_coordinated_elements`, `configuration`,
`completed` (with the `cancelled` flag), `is_consumed`, `dispose`. WinUI's members are added
(`get_for_current_view`, `cancel`, `is_scale_animation_enabled`, `set_animation_component`). The
engine is WinUI's:

- `prepare_to_animate` records the clipped and unclipped rectangles of the source in the
  coordinates of the window and freezes an image of the source (WinUI freezes a visual surface
  of the source on the next frame and keeps the source in unloading storage until then; here the
  image is rendered at once, see the deviations).
- `try_start` moves the animation to *started*; the first frame after the destination's layout
  computes its rectangles, builds the host sprite in the connected animation layer of the
  window (a container visual above the content, WinUI's `ConnectedAnimationRoot`), hides the
  source and destination elements, and starts the offset, scale, clip and cross-fade animations
  in a scoped batch whose completion ends the animation.
- The gravity configuration adds its dip (offset and scale key frames through a property set
  combined with the primary animation by expressions, as `GetEffectPropertySet` does) and a
  shadow; the direct configuration uses 200 ms and a decelerating curve; the basic one adds
  nothing.

### 5.2 Element visual properties

The compositing renderer writes `Offset`, `Size`, `Opacity`, `IsVisible`, the clip and
`TransformMatrix` of an element's visual whenever the element is rendered again, as WinUI's layout
writes the handoff visual. `Translation`, `Scale`, `CenterPoint`, `RotationAngle`, `Orientation`
and `AnchorPoint` are left to the application, except those whose facade on the visual
(`set_translation`, `set_scale`, `set_center_point`, `set_rotation`, `set_rotation_axis`) was set:
the renderer then writes them too, as WinUI writes the handoff visual from the facades. An implicit animation on `Offset` therefore animates
every layout move of the element, as in WinUI.

## 6. Status

| Phase | State | Pull request |
|---|---|---|
| 1 | in progress | draft |
| 2 | planned | |
| 3 | planned | |
| 4 | planned | |
| 5 | planned | |
| 6 | planned (scoped) | |

## 7. Deviations

Every difference from WinUI, in the format of `DEVIATIONS.md` (Representation, Behaviour,
Missing). `DEVIATIONS.md` points here for this area.

| WinUI | Port | Kind | Why | Since |
|---|---|---|---|---|
| `Visual.TransformMatrix` is a `Matrix4x4`. | A 2D `Matrix`. | Representation | The renderers of FerroUI are 2D; the property is the one the compositor already had. | phase 1 |
| `ConnectedAnimation` freezes a `CompositionVisualSurface` of the source on the next frame and keeps the source element in unloading storage until then. | The source is rendered to a bitmap when `prepare_to_animate` is called, and the bitmap is shown by a `LoadedImageSurface`. | Behaviour | A detached element's composition visual loses its content in the same batch, and the server applies every queued batch before it runs jobs, so a server-side freeze could see the detached state. The image is the same; it is taken a frame earlier. | phase 1 |
| `SetExpressionReferenceParameter(String, IAnimationObject)`. | Takes `Rc<dyn AsCompositionObject>` and is the same as `set_reference_parameter`; an element facade passes its visual. | Representation | `IAnimationObject` and `PopulatePropertyInfo` are not ported. | phase 3 |
| `InitialValueExpressions` is an `InitialValueExpressionCollection` that is itself a `CompositionObject` (`IMap<String, String>`). | A plain map owned by the animation, with the members of the map. | Representation | It needs no server counterpart or identity of its own. | phase 3 |
| The hue path of `ColorLerpHSL` is not documented. | The hue takes the short way round, and an achromatic colour takes the hue of the other colour. | Behaviour | The port had to choose. | phase 3 |
| The case of member, swizzle and parameter names in expressions is not documented. | Function names are matched ignoring case (as the expression builder of the Windows Community Toolkit writes them in lower case); member, swizzle and parameter names are matched with case. | Behaviour | Function names follow the toolkit; the rest is unverified and left as it was. | phase 3 |
| `Matrix4x4.CreateRotationX/Y/Z` and the `System.Numerics` overloads of `CreateScale`, `CreateTranslation`, `CreateRotation`, `CreateSkew` and `Lerp` of matrices are not in the expression reference. | Accepted. | Addition | They are the factories of the numeric types the expressions use; an expression valid in WinUI is valid here. | phase 3 |
| A `CompositionPropertySet` is a live object of the compositor whose entries can be animated. | The same. The parameters of an animation are a property set without a server counterpart, copied when the animation starts (the upstream behaviour). | Representation | See `DEVIATIONS.md` for the difference from upstream. | phase 1 |
| The `Completed` handler of a batch receives `(sender, args)`. | The args only. | Representation | The event convention of FerroUI; the batch need not be kept alive by the application. | phase 1 |
| One commit batch per frame and batch type. | One per frame, which waits for the union of the requested types. | Behaviour | Animations started before `GetCommitBatch` must be included. | phase 1 |
| Whether expression animations belong to a batch is not documented. | They belong to no batch, not even an `InfiniteAnimation` one: they never end. | Behaviour | Unverified against WinUI. | phase 1 |
| The `Effect` batch type waits for effects. | Accepted and never waited on. | Missing | No effects yet (phase 6). | phase 1 |
| A single-frame initial or final step of `StepEasingFunction` lasts one frame. | It lasts no time: it shows at the start or the end only. How the duration is split between steps is the port's reading. | Behaviour | An easing function does not know frames. | phase 1 |
| Easing functions are `CompositionObject`s. | Plain `IEasing` values (`Easing`); the `owner` argument of the factories is not used. | Representation | Key frames take `Easing`. | phase 1 |
| Stopping `Offset.X` while `Offset` is animated as a whole is not documented. | Does nothing. Starting `Offset.X` while `Offset.XY` runs replaces the whole `XY` channel. | Behaviour | Simplification of an undocumented case. | phase 1 |
| Animations run on a visual that is not in a tree. | They wait until the server object is active (attached to a composition target), and so does the `Completed` of their batch. | Behaviour | The upstream compositor activates animations with the tree; a property set is always active. | phase 1 |
| `InteractionTracker` runs in the compositor. | It runs on the UI thread and ticks with `MediaContext::request_animation_frame` while in inertia or a custom animation; custom animations and modifier expressions are evaluated there, and a reference parameter that is not a property set reads as missing. | Behaviour | The compositor of FerroUI has no input and no callbacks to the UI thread. Expressions that bind visuals to the tracker run on the compositor as usual. | phase 4 |
| The values of an `InteractionTracker` are its properties; its `Properties` belongs to the application. | The values live in the tracker's property set under the WinUI names. | Representation | A reference to the tracker resolves to that live property set. | phase 4 |
| The inertia decay rate and curve are not documented. | `v(t) = v0 * (1 - rate)^t`, resting at `p + v / ln(1 / (1 - rate))`; inertia motion `target + (start - target) * r^t`; `InteractionTrackerInertiaMotion.Motion` is the value of the axis each frame; overpan is a rubber band capped at 120 px; a custom animation that ends out of bounds returns through inertia; scale custom animations are clamped. | Behaviour | The port had to choose; the literal `rate^t` moves about 19.5 times the velocity. | phase 4 |
| Owner callbacks of the tracker are raised by the compositor; `IsFromBinding`. | Queued and delivered after the change, never nested; input-driven callbacks carry the last request id; `IsFromBinding` is always `false`. | Behaviour | No re-entrancy; tracker binding is not ported. | phase 4 |
| `VisualInteractionSource` receives input from the compositor after `TryRedirectForManipulation`. | It follows the routed pointer events of the element of the visual (handled ones too), captured to the element, for any pointer type; the element is found from the visual or given to `create_for_element`. The mouse wheel scrolls 50 px per notch and Ctrl+wheel scales by 1.1. Chaining modes are stored only; the input values of a source cannot be referenced from expressions. | Behaviour | The compositor of FerroUI has no input. | phase 4 |
| `InteractionTrackerInertiaNaturalMotion`, `ConfigureVector2PositionInertiaModifiers`, tracker binding, `InteractionTrackerPositionUpdateOption`, `AdjustPositionX/YIfGreaterThanThreshold`, pinch and touchpad redirection. | Not ported. | Missing | Natural motion animations come with phase 3; FerroUI has no touchpad input and no two-pointer recognizer. | phase 4 |
| `Visual.Clip` takes a `CompositionClip`. | `composition_clip` / `set_composition_clip`; it may be set together with the geometry clip of the upstream compositor (`clip`), and the two intersect. | Representation | `clip` is the upstream geometry clip already. | phase 1 |
| The reference box of `CompositionClip.AnchorPoint` is not documented. | Relative to the size of the visual. | Behaviour | The port had to choose. | phase 1 |
| `RotationAngleInDegrees` of clips and of the surface brush is an animatable property. | A view of `RotationAngle`, not an animation target. | Missing | Only the radians are stored. | phase 1 |
| `CompositionSurfaceBrush` is a brush like the others. | It is painted by the sprite (and later shape) visual itself, not handed to the drawing context; the upstream `Transform` and `RelativeTransform` of composition brushes are not applied to it; in a draw list it draws nothing. | Representation | The drawing contexts have no brush for a composition surface. | phase 1 |
| `BitmapInterpolationMode` has separate minification, magnification and mipmap filters. | Mapped to the single bitmap quality of the drawing context; `Linear` leaves the quality of the context unchanged. | Behaviour | The drawing context has one setting. | phase 1 |
| `DropShadow` follows the alpha of the content or its `Mask`; its `Offset.Z` matters with lights. | The box shadow of the visual's rectangle; `Mask` and `SourcePolicy` are stored and not used; the Z of the offset is ignored; there is no separate `CompositionShadow` type. | Missing | Phase 5. | phase 1 |
| `CompositionVisualSurface` follows its source wherever it is. | Changes of the source are followed while the source is attached to a composition target; a surface that shows an ancestor of its own sprite skips the inner drawing of itself. `freeze()` (not public in WinUI) renders the source into a layer once, with the bounds of the last frame. | Behaviour | Live updates hook into the update pass of the target. | phase 1 |
| `LoadedImageSurface` loads from a URI or a stream, asynchronously, with a decode size and `LoadCompleted`. | Made from a `Bitmap`, or decoded from a stream at once; `load_status` instead of the event; no URI loading and no decode size. | Missing | The surface is backed by a bitmap. | phase 1 |
| A WinUI clip limits the hit testing of composition visuals. | It does not. | Missing | The hit testing of the compositor knows the geometry clip only. | phase 1 |
| `ScalarTransition` and `Vector3Transition` are classes: a change of `Duration` after the assignment reaches the element. | Copyable values, copied when assigned; a change is made by assigning again. | Representation | No identity or change notification is needed. | phase 3 |
| `UIElement.RotationAxis` rotates in 3D. | An axis other than Z is written to `Orientation`, of which the 2D renderers show the projection; a zero axis counts as Z. | Behaviour | The renderers are 2D. | phase 3 |
| `UIElement.TransformMatrix` is applied after the other facades. | Its 2D affine part (M11, M12, M21, M22, M41, M42) is applied after the render transform and before `Scale`, the rotation and `Translation`; the 3D part is ignored. | Behaviour | The composition visual applies its `TransformMatrix` first; folding `Scale` and the rotation into it would break their animations. | phase 3 |
| A facade transition and an animation of the same property of the handoff visual cannot be combined. | While any transition is set, the implicit animations of the facades replace those the application set on the element visual. | Behaviour | A visual has one implicit animation collection. | phase 3 |
| `Vector3Transition.Components` is not documented in detail; the easing of transitions is not documented. | Excluded components jump to their final value at the start; a transition with no components does nothing. The transitions use the default easing of the compositor. | Behaviour | The port had to choose. | phase 3 |
| `SetImplicitHideAnimation` keeps an element removed from the tree in unloading storage until the animation completes. | The animation runs on an image of the element taken when it is removed. | Behaviour | A detached element loses its composition visual and its content in the same batch. | phase 1 |
