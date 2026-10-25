# Android Material 3 Expressive theme notices

## AndroidX / Material Design

Material numeric tokens, component geometry, state definitions, motion constants,
ripple timing and progress algorithms are adapted from AndroidX Compose Material3
1.5.0-beta02, commit `e4d933c98df7baa7ea2b83eea6e086915961e0ff`:
<https://android.googlesource.com/platform/frameworks/support/+/e4d933c98df7baa7ea2b83eea6e086915961e0ff/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/>.

Copyright The Android Open Source Project. Licensed under Apache License 2.0;
see `LICENSE-ANDROID`. These definitions have been translated and adapted for
FerroUI's Rust controls and XAML resource system. This is an independent port,
not a Google product or an Android platform implementation.

`material-colors` is used for CAM16/HCT dynamic palettes under MIT OR Apache-2.0.
Its version and algorithm baseline are recorded in the manifest and port guide.

## Google Material Color Utilities

`dynamic_color_spec.rs` ports the phone palette, color-role, contrast, and
tone-constraint rules from Google Material Color Utilities, commit
`bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3`:
<https://github.com/material-foundation/material-color-utilities/tree/bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3/typescript>.
Source files are `dynamiccolor/color_spec_2021.ts`, `color_spec_2025.ts`,
`dynamic_scheme.ts`, `dynamic_color.ts`, and `palettes/tonal_palette.ts`.
Copyright 2021, 2022, 2025 Google LLC. Licensed under Apache License 2.0; see
`LICENSE-ANDROID`. The reference fixtures execute the original TypeScript;
their source hashes and reproducible generation command are recorded in
`tests/fixtures/dynamic-color-provenance.json` and the dynamic-color port guide.

## Roboto

`Assets/Roboto.ttf` is distributed under the SIL Open Font License 1.1, included
as `Assets/OFL-Roboto.txt`. The original variable font is unmodified. The regular,
medium and bold files are static instances generated from it with fontTools so
font backends need not synthesize weights. Source
revision, URL and SHA-256 are recorded in `scripts/material-theme/font-provenance.json`.

## WinUI / Avalonia structural heritage

The initial control-template contracts, resource aliases, build registration,
layout primitives and converters were copied from FerroUI's WinUI theme. The
following original notices remain applicable to that inherited material.


The markup documents of this crate are ported from the WinUI 3 theme of
microsoft-ui-xaml (<https://github.com/microsoft/microsoft-ui-xaml>, branch
`winui3/main`, commit `7e4b0c2096be402f51cc8f7d203ebf67cff183d0`):
`dxaml/xcp/dxaml/themes/generic.xaml`, the `*_themeresources.xaml` documents and
the default style documents of `controls/dev`, and the density documents of
`controls/dev/dll/DensityStyles`. Each ported document names its source documents
in its header comment; `docs/porting/winui-theme.md` at the root of the repository
describes the port. The resource documents under `Resources/` are converted by
`scripts/winui-theme/convert.py`; the control documents under `Controls/` are
ported by hand. `Converters/corner_radius_to_thickness_converter.rs` is a Rust port
of the class `CornerRadiusToThicknessConverter` of the same repository
(`controls/dev/Common/CornerRadiusToThicknessConverter.idl`, `.h` and `.cpp`), and
`Primitives/info_bar_panel.rs` a Rust port of the class `InfoBarPanel`
(`controls/dev/InfoBar/InfoBarPanel.cpp`, `.h` and `InfoBar.idl`), and
`Primitives/tab_view_equal_width_panel.rs` a Rust port of the sizing of the tabs of
`TabView` in its `Equal` width mode (`TabView::UpdateTabWidths` of
`controls/dev/TabView/TabView.cpp`, `.h` and `TabView.idl`).

## microsoft-ui-xaml

The ported documents are subject to the following license:

```
MIT License

Copyright (c) Microsoft Corporation. All rights reserved.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE
```

## Avalonia

The Rust code of the crate (the theme class, the accent colours, the build script
and the tests) and the templates of the controls that WinUI 3 has no style for
(the window, popup and adorner hosts) are patterned on the Fluent theme of
FerroUI (`src/FerroUI.Themes.Fluent`), which is converted from the Fluent theme of
the upstream project, Avalonia (MIT; see the project notice reproduced below and `NOTICE.md` at
the root of the repository). The outlines `GlyphGeometryE73E` and
`GlyphGeometryE894` of `Resources/Glyphs.xaml` are the check mark and the clear
button outlines of that theme.

## Symbol font

The templates of microsoft-ui-xaml show some glyphs of the Segoe Fluent Icons
font of Windows. The font is not part of this crate and is not redistributed:
`Resources/Glyphs.xaml` draws each glyph the port needs as a geometry.


---

## Project attribution reproduced for crate distributions

# NOTICE

FerroUI is a Rust port of the Avalonia UI framework. Its architecture, public API shape, algorithms and a large part of its behaviour are derived from the Avalonia sources, and some components contain directly imported upstream code. This file records the upstream projects and their licenses. Component-level notices with file mappings live next to the code they cover.

## Avalonia

- Project: <https://github.com/AvaloniaUI/Avalonia>
- License: MIT
- Used for: the whole framework design and implementation that the crates under `src/` port (base library, controls, markup, rendering and platform backends), the unit tests ported alongside them, and the native macOS backend imported under `native/FerroUI.Native/` (see `native/FerroUI.Native/NOTICE.md`).

```
The MIT License (MIT)

Copyright (c) AvaloniaUI OÜ
All Rights Reserved

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

Avalonia itself incorporates code from other projects (for example WPF-derived layout and text code under the MIT license from the .NET Foundation, and Chromium-derived snippets in the native macOS backend); those notices are carried in the upstream `NOTICE.md` and, where the corresponding code is ported or imported here, in the component-level notices.

## XamlX

- Project: <https://github.com/kekekeks/XamlX>
- License: MIT, Copyright (c) 2019 Nikita Tsukanov
- Used for: the XAML compiler front end ported under `external/XamlX/` (see `external/XamlX/NOTICE.md`).

## MicroCom

- Project: <https://github.com/kekekeks/MicroCom>
- License: MIT
- Used for: the design of the COM-style interop runtime and IDL code generator under `src/FerroUI.MicroCom/` and `src/tools/MicroCom.CodeGenerator/` (see the `NOTICE.md` files there).

## .NET runtime

- Project: <https://github.com/dotnet/runtime>
- License: MIT, Copyright (c) .NET Foundation and Contributors
- Used for: the masked text engine `src/FerroUI.Controls/utils/masked_text_provider.rs`, derived from `System.ComponentModel.MaskedTextProvider` (the class the upstream masked text box is built on); and `src/FerroUI.DependencyInjection` plus `tests/FerroUI.DependencyInjection.Tests`, derived from Microsoft.Extensions.DependencyInjection, its abstractions and specification tests at `v10.0.13` (`6aa72daa24f66c01c3d7b6a32519e9d21396deec`). See `src/FerroUI.DependencyInjection/NOTICE.md` and `docs/porting/dependency-injection.md`.

## microsoft-ui-xaml (WinUI 3)

- Project: <https://github.com/microsoft/microsoft-ui-xaml> (branch `winui3/main`, commit `7e4b0c2096be402f51cc8f7d203ebf67cff183d0`)
- License: MIT, Copyright (c) Microsoft Corporation
- Used for: the WinUI theme `src/FerroUI.Themes.Material`, whose resource and control theme documents are ported from the theme of WinUI 3 (`generic.xaml` and the theme resources of the controls library; see `src/FerroUI.Themes.Material/NOTICE.md`, which holds the license text, and `docs/porting/winui-theme.md`). Also for the Charts library `src/FerroUI.Controls.Charts`, whose conformance tests and the ChartsDemo scenarios are adapted from the public Charts tests and ChartApp samples at commit `9427b7216337ea3d7b19ac32d48c327286ee7138` (see `src/FerroUI.Controls.Charts/NOTICE.md` and `docs/porting/winui-charts.md`).

## Reactive Extensions for .NET (Rx.NET)

- Project: <https://github.com/dotnet/reactive> (`Rx.NET/Source`, commit `89e05b8`)
- License: MIT, Copyright (c) .NET Foundation and Contributors
- Used for: `src/FerroUI.Reactive` (a port of System.Reactive), `src/FerroUI.Reactive.Testing` (a port of Microsoft.Reactive.Testing) and their unit tests in `tests/FerroUI.Reactive.Tests`, ported from `Tests.System.Reactive` (`docs/porting/reactive.md`).

## Splat

- Project: <https://github.com/reactiveui/splat> (commit `39466de`)
- License: MIT, Copyright (c) ReactiveUI 2018 - 2025
- Used for: the optional `src/FerroUI.DependencyInjection/splat.rs` adapter from Splat.Microsoft.Extensions.DependencyInjection (same commit), and `src/FerroUI.Splat` (a port of Splat.Core, Splat.Logging, Splat.Builder, and the mode detection and feature usage tracking of Splat) and its unit tests in `tests/FerroUI.Splat.Tests`, ported from Splat.Tests, Splat.Logging.Tests and Splat.Builder.Tests (`docs/porting/reactive.md`).

## DynamicData

- Project: <https://github.com/reactivemarbles/DynamicData> (`src`, commit `65016e5`)
- License: MIT, Copyright (c) Roland Pheasant 2011-2022
- Used for: `src/FerroUI.DynamicData` (a port of DynamicData) and its unit tests in `tests/FerroUI.DynamicData.Tests`, ported from `DynamicData.Tests` (`docs/porting/reactive.md`).

## ReactiveUI

- Project: <https://github.com/reactiveui/ReactiveUI> (`src/ReactiveUI.Core`, `src/ReactiveUI.Shared`, `src/Shared`, `src/ReactiveUI.Testing`, `src/ReactiveUI.Testing.Reactive`, commit `721f544`)
- License: MIT, Copyright (c) .NET Foundation and Contributors
- Used for: `src/FerroUI.ReactiveUI` (a port of ReactiveUI in its System.Reactive flavour), `src/FerroUI.ReactiveUI.Testing` (a port of ReactiveUI.Testing) and their unit tests in `tests/FerroUI.ReactiveUI.Tests` and `tests/FerroUI.ReactiveUI.Testing.Tests`, ported from ReactiveUI.Tests, ReactiveUI.Builder.Tests, ReactiveUI.Splat.Tests and ReactiveUI.Testing.Tests (`docs/porting/reactive.md`).

## ReactiveUI.Binding

- Project: <https://github.com/reactiveui/ReactiveUI.Binding.SourceGenerators> (`src/ReactiveUI.Binding.Shared`, commit `d1e1b65`)
- License: MIT, Copyright (c) ReactiveUI Association Incorporated 2019 - 2026
- Used for: the run time of the property observation, output properties, bindings, converters, interactions and view location in `src/FerroUI.ReactiveUI/binding`, and their unit tests in `tests/FerroUI.ReactiveUI.Tests/tests/binding`, ported from ReactiveUI.Binding.Tests and ReactiveUI.Binding.GeneratedCode.Tests (`docs/porting/reactive.md`).

## ReactiveUI.Primitives

- Project: <https://github.com/reactiveui/Primitives> (`src/ReactiveUI.Primitives.Reactive`, `src/Primitives.Platform.Reactive.Shared`, `src/Primitives.Core`, commit `18e8a85`)
- License: MIT, Copyright (c) 2019-2026 ReactiveUI Association Incorporated
- Used for: the System.Reactive extras that ReactiveUI calls, in `src/FerroUI.ReactiveUI/primitives` and in the delayed notifications of `src/FerroUI.ReactiveUI/reactive_object`.

## ReactiveUI.Avalonia

- Project: <https://github.com/reactiveui/ReactiveUI.Avalonia> (`src/ReactiveUI.Avalonia`, `src/Shared`, commit `26f23b5`), with `ReactiveUI.Binding.Avalonia.Shared` of ReactiveUI.Binding (commit `d1e1b65`) and the scheduler of `ReactiveUI.Primitives.Avalonia.Reactive` (commit `18e8a85`)
- License: MIT, Copyright (c) 2019-2026 ReactiveUI and Avalonia Teams, and Contributors
- Used for: `src/FerroUI.ReactiveUI.Controls` (the integration of ReactiveUI with FerroUI) and its unit tests, ported from ReactiveUI.Avalonia.Tests and ReactiveUI.Binding.Avalonia.Tests (`docs/porting/reactive.md`); and `samples/ReactiveUIExample` (the port of its example application, `src/examples`, and its tests).

## ReactiveUI.Samples

- Project: <https://github.com/reactiveui/ReactiveUI.Samples> (`wpf/getting-started/ReactiveDemo`, commit `8e7ca27`)
- License: MIT, Copyright (c) .NET Foundation and Contributors
- Used for: `samples/ReactiveDemo` (the port of `ReactiveDemo`, the getting-started sample of the ReactiveUI documentation; see `samples/ReactiveDemo/NOTICE.md`).

## RemoteViewing

- Project: <https://github.com/quamotion/remoteviewing> (package `Quamotion.RemoteViewing` 1.1.211)
- License: BSD-2-Clause, Copyright (c) 2013 James F. Bellinger
- Used for: the RFB (VNC) server of the VNC server of the headless platform, ported under `src/Headless/FerroUI.Headless.Vnc/remote_viewing/`: the parts of the library the upstream project uses (see `src/Headless/FerroUI.Headless.Vnc/NOTICE.md`, which holds the license text).

## Code under other licenses

Some ported files derive from sources that the upstream project took from third parties under licenses other than MIT. They keep their original header lines, and the component-level notice lists them with the full license text:

- Silverlight Toolkit, Microsoft Public License (Ms-PL): the `AutoCompleteBox` sources and the selection adapters under `src/FerroUI.Controls/` (see `src/FerroUI.Controls/NOTICE.md`). Ms-PL requires that source distributions of those portions stay under Ms-PL.
- WPF and WinUI (MIT, .NET Foundation / Microsoft Corporation): see `src/FerroUI.Controls/NOTICE.md`, `src/FerroUI.Controls.ColorPicker/NOTICE.md` and the file headers in `src/FerroUI.Base/media/`.
- Inter typeface (SIL Open Font License 1.1): the font files embedded by `src/FerroUI.Fonts.Inter` (see `src/FerroUI.Fonts.Inter/NOTICE.md`).
- Roboto typeface (Apache License 2.0, Copyright 2011 Google Inc.): the font file of the about dialog embedded by `src/FerroUI.Dialogs` (see `src/FerroUI.Dialogs/NOTICE.md`).
- Test fonts of `ferroui-base` (SIL Open Font License 1.1: Inter, Adobe Blank 2; Apache License 2.0: two files derived from WenQuanYi Micro Hei): used by its tests only (see `src/FerroUI.Base/NOTICE.md`).
- wayland-protocols (MIT-style): popup positioner documentation and flag names, see `src/FerroUI.Controls/NOTICE.md`.

## Third-party crates

Rust dependencies are used under their own licenses as declared in their crate metadata; they are not vendored in this repository. Direct dependencies: `skia-safe` (MIT), `harfbuzz-sys` (MIT), `bitflags`, `paste`, `cc`, `roxmltree`, `time` (MIT OR Apache-2.0), `rust_decimal` (MIT) and `sysinfo` (MIT); for the VNC server of the headless platform `flate2`, `jpeg-encoder`, `des` and `getrandom` (MIT OR Apache-2.0).

The Vello render backend (`src/Vello/FerroUI.Vello`) links crates of the Vello project and their companions: `kurbo`, `peniko`, `vello_cpu` (with `vello_common`), `linesweeper` and `png`, each under the Apache License 2.0 or the MIT license at the choice of the user. The versions, what each is used for and the copyright lines are in `src/Vello/FerroUI.Vello/NOTICE.md`.

The Vulkan platform contracts (`src/FerroUI.Vulkan`) use `ash` 0.38.0 (MIT OR Apache-2.0, Copyright (c) 2016 ASH contributors) for the types, constants and function tables of Vulkan, with `libloading` (ISC) to open the Vulkan loader of the system at run time.
