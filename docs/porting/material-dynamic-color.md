# Material dynamic color

`ferroui-themes-material` supports all 48 AndroidX `ColorScheme` roles through
`MaterialColorScheme`. There are two sources of dynamic color: locally generated
HCT seed schemes and lossless imports of Android platform colors. Both feed the
same live Material resources, state-layer brushes, and compatibility resources.

## Generate a seed scheme

```rust
use ferroui_base::media::Color;
use ferroui_base::styling::ThemeVariant;
use ferroui_themes_material::{MaterialColorScheme, MaterialTheme};

let theme = MaterialTheme::new();
let seed = Color::from_uint32(0xff00639b);
theme.set_color_scheme(ThemeVariant::light(), &MaterialColorScheme::from_seed(seed, false));
theme.set_color_scheme(ThemeVariant::dark(), &MaterialColorScheme::from_seed(seed, true));

// Material Color Utilities contrast: -1 minimum, 0 standard, 0.5 medium, 1 high.
let high_contrast = MaterialColorScheme::from_seed_color(seed, true, 1.0)?;
theme.set_color_scheme(MaterialTheme::high_contrast(), &high_contrast);
# Ok::<(), ferroui_themes_material::MaterialDynamicColorError>(())
```

Seed generation uses the CAM16/HCT gamut solver, tonal palettes, tone-delta
constraints, and dynamic contrast curves in the exactly pinned Rust
[`material-colors` 0.4.2](https://docs.rs/material-colors/0.4.2/material_colors/).
It does not blend the seed with black or white in RGB. Source alpha is ignored,
and generated roles are opaque. Invalid or nonfinite contrast inputs return a
diagnostic rather than silently changing the requested contrast.

The palette style is independent of the theme's expressive motion scheme:

```rust
use ferroui_themes_material::{MaterialDynamicColorOptions, MaterialPaletteStyle};

let scheme = MaterialColorScheme::from_seed_with_options(seed,
    MaterialDynamicColorOptions {
        dark: false,
        contrast_level: 0.5,
        style: MaterialPaletteStyle::Expressive,
    })?;
```

Supported styles are `Monochrome`, `Neutral`, `TonalSpot` (default), `Vibrant`,
`Expressive`, `Fidelity`, `Content`, `Rainbow`, and `FruitSalad`. These correspond
to the Material Color Utilities variants. The full algorithm and contrast
meaning are described in the upstream
[scheme guide](https://github.com/material-foundation/material-color-utilities/blob/main/dev_guide/creating_color_scheme.md).

### Algorithm version is explicit

`MATERIAL_COLOR_UTILITIES_VERSION` identifies the implementation used for seed
colors. Version 0.4.2 supports this repository's Rust toolchain; the newer 0.5.0
release declares Rust 1.97. The dependency is pinned rather than changing seed
colors implicitly in a patch update.

This does **not** imply that a given seed generates exactly the same scheme as
every Android release, OEM wallpaper algorithm, or newer Material Color Utilities
revision. The utility has evolved its role curves. For example, the pinned
implementation's light `onPrimaryContainer` uses the tone-10 convention, while
Google's JavaScript 0.3.0 uses a newer tone-30 convention. The current AndroidX
baseline and API 31 mapping also use tone 10. Default-contrast seed goldens are
checked independently against Google's JavaScript utilities 0.2.7; high-contrast
schemes are additionally checked using their rendered sRGB contrast ratios.
To match actual device colors, use the platform imports below.

## Native Android system-color bridge

Enable the optional feature in the Android application's dependencies:

```toml
[dependencies]
ferroui-themes-material = { version = "0.4", features = ["android-dynamic-color"] }
```

Create the theme after the FerroUI Android application is initialized, then import
the device palette:

```rust
use ferroui_themes_material::{MaterialAndroidColorSource, MaterialTheme};

let theme = MaterialTheme::new();
let source = theme.use_android_dynamic_colors()?;
match source {
    MaterialAndroidColorSource::Android34ColorRoles => {
        // Current system role colors, including the device's contrast adjustments.
    }
    MaterialAndroidColorSource::Android31TonalPalette => {
        // Android 12/13 wallpaper palettes and AndroidX-derived neutral tones.
    }
}
// Add theme.as_style() to the application's styles as usual.
# Ok::<(), ferroui_themes_material::MaterialAndroidColorError>(())
```

The bridge obtains `FerroAndroidApplication::try_context()` and reads public
Android color resources through the backend's safe Java interoperability API.
Applications supply no JNI code, Java helper, AndroidX dependency, or extra
resource package. The optional `ferroui-android` dependency is restricted to
Android targets; enabling the feature on desktop does not link an Android
renderer. The same method remains callable on desktop and returns
`MaterialAndroidColorError::UnsupportedPlatform`.

`use_android_dynamic_colors` validates both complete schemes before replacing the
light and dark theme dictionaries; it also updates the default dictionary to the
imported dark scheme, consistent with the theme's default fallback. Missing
resources, unsupported Android releases, or an application that has not yet been
initialized return a descriptive error and retain the existing colors. The
method reports which resource contract it used. On API 34+ an incomplete direct
role scheme is an error; it does not silently downgrade to the older tonal
algorithm and discard system contrast decisions.

### Refresh and subscription lifetime

The method subscribes to the existing
`IPlatformSettings::color_values_changed` notification using a weak theme
reference. A second call replaces that subscription. Dropping the theme or
calling `stop_android_dynamic_colors()` disposes it; stopping retains the last
successfully imported colors.

```rust
// Re-read after an application-specific Android resource-overlay change.
let source = theme.refresh_android_dynamic_colors()?;

// Keep the current palette, but stop following subsequent platform notifications.
theme.stop_android_dynamic_colors();
# Ok::<(), ferroui_themes_material::MaterialAndroidColorError>(())
```

Automatic refresh follows the notifications delivered by FerroUI's Android
platform settings. That backend currently observes Android configuration changes
and publishes when its reported theme, contrast preference, or accent colors
change. The theme does not install an additional wallpaper receiver or an
Android `UiModeManager` contrast listener. A custom resource overlay or a contrast
change that leaves the backend's reported values unchanged therefore needs an
explicit `refresh_android_dynamic_colors()` call from the host's notification.
If an automatic refresh finds an incomplete palette during a configuration
transition, it retains the last complete palette. Explicit refresh returns the
error for the application to inspect.

### Native resource fidelity

On API 34+, the bridge follows AndroidX's
[`values-v34/styles.xml`](https://android.googlesource.com/platform/frameworks/support/+/e4d933c98df7baa7ea2b83eea6e086915961e0ff/compose/material3/material3/src/androidMain/res/values-v34/styles.xml)
mapping to public `system_*_light` and `system_*_dark` colors. Inverse roles read
the opposite variant, fixed roles use the unqualified `system_*_fixed` resources,
and surface tint reads the corresponding primary. All imported ARGB values are
preserved, including the system's current contrast adjustments.

On API 31–33 it reads all 65 actual system palette colors according to
[`values-v31/styles.xml`](https://android.googlesource.com/platform/frameworks/support/+/e4d933c98df7baa7ea2b83eea6e086915961e0ff/compose/material3/material3/src/androidMain/res/values-v31/styles.xml).
The eleven additional tones in each neutral family are calculated from that
family's tone-40 color by the pinned AndroidX float CAM16 and HCT path in
`android_palette_math.rs`. Its float32 rounding boundaries and HCT lookup table
are preserved separately from the locally generated seed algorithm. Actual
device tones pass through unchanged; missing system resources are errors.

## Android 12–13: custom tonal-palette import

`MaterialDynamicPalette` holds primary, secondary, tertiary, neutral, and
neutral-variant palettes. `from_android31_colors` requests the exact tones used
by AndroidX's `dynamicTonalPalette` function, and `scheme_android31` maps those
tones to roles using the AndroidX API 31–33 implementation.

```rust
use ferroui_themes_material::{MaterialDynamicPalette, MaterialPaletteFamily};

// For a custom host or a captured reference palette. This closure is illustrative;
// read_androidx_tone is application/platform code, not a FerroUI API.
let palette = MaterialDynamicPalette::from_android31_colors(|family, tone| {
    read_androidx_tone(family, tone)
})?;
let light = palette.scheme_android31(false)?;
let dark = palette.scheme_android31(true)?;
theme.set_color_scheme(ThemeVariant::light(), &light);
theme.set_color_scheme(ThemeVariant::dark(), &dark);
```

The three accent palettes require tones 0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95,
99, and 100. Both neutral palettes additionally include 4, 6, 12, 17, 22, 24, 87,
92, 94, 96, and 98. AndroidX calculates those additional neutral tones using
`Color.setLuminance` from the corresponding tone-40 system color. This lower-level
adapter expects the supplied palette to include those calculated values; the
native bridge above provides them automatically. The data-only import never
guesses missing tones or substitutes the seed-generation algorithm for the
platform's calculation. `ANDROID_SYSTEM_TONES` and `ANDROID_NEUTRAL_TONES` expose
the requested sets.

The role mapping follows
[`DynamicTonalPalette.android.kt`](https://android.googlesource.com/platform/frameworks/support/+/e4d933c98df7baa7ea2b83eea6e086915961e0ff/compose/material3/material3/src/androidMain/kotlin/androidx/compose/material3/DynamicTonalPalette.android.kt)
at the theme's pinned AndroidX revision. The neutral-variant choice is
intentional: AndroidX uses it for API 31–33 to avoid the older platform's
lower-chroma neutral palette.

| Role group | Light tone | Dark tone | Palette |
| --- | ---: | ---: | --- |
| Primary, secondary, tertiary | 40 | 80 | Corresponding accent |
| On accent | 100 | 20 | Corresponding accent |
| Accent container | 90 | 30 | Corresponding accent |
| On accent container | 10 | 90 | Corresponding accent |
| Fixed accent / dim | 90 / 80 | 90 / 80 | Corresponding accent |
| On fixed / variant | 10 / 30 | 10 / 30 | Corresponding accent |
| Background, surface | 98 | 6 | Neutral variant |
| On background, on surface | 10 | 90 | Neutral variant |
| Surface variant / on variant | 90 / 30 | 30 / 80 | Neutral variant |
| Inverse surface / on inverse | 20 / 95 | 90 / 20 | Neutral variant |
| Outline / outline variant | 50 / 80 | 60 / 30 | Neutral variant |
| Surface bright / dim | 98 / 87 | 24 / 6 | Neutral variant |
| Surface container lowest | 100 | 4 | Neutral variant |
| Surface container low | 96 | 10 | Neutral variant |
| Surface container | 94 | 12 | Neutral variant |
| Surface container high | 92 | 17 | Neutral variant |
| Surface container highest | 90 | 22 | Neutral variant |
| Surface tint | 40 | 80 | Primary |
| Inverse primary | 80 | 40 | Primary |
| Scrim | 0 | 0 | Neutral variant |

As in the AndroidX function, the four error roles retain the baseline light/dark
scheme values. `MaterialTonalPalette::from_tones` also permits sparse imports for
application-specific adapters; using a sparse palette in a scheme fails with
`MissingTone` when a required tone is absent. Imported values, including alpha,
are preserved exactly.

`MaterialTonalPalette::from_hct` and `MaterialDynamicPalette::from_seed` generate
complete local palettes for applications that need direct tone access. They are
explicit alternatives to importing device colors.

## Android 14 and newer: custom direct-role import

API 34 and newer expose complete scheme roles. AndroidX reads its
`mc3_light_scheme` and `mc3_dark_scheme` style attributes directly. Reconstructing
these roles from the thirteen system tones would discard platform contrast and
palette decisions. Use `MaterialColorScheme::from_android_system_roles` instead:

```rust
let scheme = MaterialColorScheme::from_android_system_roles(true, |role| {
    // Resolve the matching mc3_dark_scheme attribute in the platform bridge.
    read_android_scheme_role(role)
})?;
theme.set_color_scheme(ThemeVariant::dark(), &scheme);
```

All 43 roles supplied by AndroidX are required. The four error roles and scrim
retain their baseline defaults when absent, just as in AndroidX; explicit values
for these five roles are accepted and preserved. This path can import the
platform's high-contrast scheme without applying a second contrast adjustment.

These data adapters remain available without the native bridge feature. They
support custom Android hosts, captured reference colors, and applications on
other platforms. A complete Material Theme Builder scheme can also be supplied
through `MaterialColorScheme::new`, or generated from a seed with explicit options.

## Validation

The module tests cover all 48 default-contrast seed colors in light and dark mode
against independent Google utility goldens, every palette variant, primary
foreground/background contrast at contrast +1 across saturated and achromatic
seeds, invalid inputs, exact imported colors, fixed-role mode invariance,
neutral-variant surface mapping, missing-tone errors, and API 34 role completeness.
Native bridge tests exercise public system-resource names, inverse and fixed-role
selection, lossless imports, complete-before-install behavior, API 31 system-tone
requests, and added neutral-tone generation. The CAM16 port has reference primary
coordinates, endpoint, luminance, and gamut checks. These deterministic tests do
not replace device testing of Android resource and configuration notifications.
The theme's resource tests separately verify that replacing a scheme updates the
Material resources consumed by controls.
