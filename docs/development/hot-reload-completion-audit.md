# Hot reload completion audit

Audited on 2026-10-20. The goal remains
active. This checklist fixes the completion scope; historical validation notes
retain their original source revisions.

## Implemented, with recorded verification

| Requirement | Current evidence and limits |
| --- | --- |
| Runtime XAML and last-good replacement | Session, overlay, detached preparation, reversible content/component/panel adapters and failure tests exist. Native catalog probes ran on all three platforms at recorded revisions. |
| Sessions and build/loading integration | Manifest, persistent/scoped populate hooks, weak targets, document buffers, directory polling/debounce, superseded revisions, transformed-document cache and debug/release gates exist. Build handler contracts include signatures and conservative unresolved-type admission, verified by build/engine tests on macOS/Linux and generated-manifest catalog tests on macOS. |
| Stable windows, roots and view models | Native identity probes and lifecycle tests cover supported adapters. Root literal/static-resource/binding ownership supports removal and rollback. |
| Compatible UI state | Focus, text/caret/selection, scroll and selection-model tests exist. Application state callbacks and queued-revision state preservation are implemented. |
| Cached controls and subscriptions | Transactional refresh hooks and the Expander sample exercise rollback, disposal and retired-tree release. Ten catalog pages are registered. This is representative integration, not all-page support. |
| Included resources/styles and static consumers | Registered application containers discover direct live includes; dependency revisions prepare resources and consumers coherently. Resource/theme probes and include lifetime tests exist. Standard inline component ownership remains incomplete below. |
| Explicit Rust patching | Real Subsecond patches update existing callback subscriptions, with stable framework identity, conservative admission and controlled application. Native patch evidence exists for macOS, Linux and Windows at recorded revisions. |
| Supervisor and restart | The dedicated sample owns admission/build/application, retains the old process on failed replacement builds and restores versioned application/window/navigation state. Native supervisor evidence exists. |
| Combined updates and measurements | Combined XAML/Rust acceptance and stage/save-to-render measurements exist for a small fixture and representative catalog pages. These are functional baselines, not statistical performance guarantees. |

## Required completion work

1. **Close standard component integration gaps.** The shared component/panel
   preparation currently rejects XAML event handlers.
   Inline Resources and Styles now have explicit ownership adapters, including deferred
   template context transfer and application-style ordering; mixed application/XAML dictionary provider trees still require
   a composed preview adapter. Implement the remaining ownership
   adapters and exercise standard handlers and resource/style replacement through
   actual component-loading paths. Complete state transfer for compatible, uniquely
   identified template parts; the current logical-child capture does not cover
   replacement root-template parts. Preserve the documented restart boundary for
   unsupported custom mutations and irreversible user code.
2. **Freeze a final implementation revision and close acceptance.** Reuse existing
   evidence where behavior and inputs are unchanged, but run the final relevant
   native XAML, Rust, combined, restart, lifecycle and release-boundary gates on
   macOS, Windows and Linux. Fix failures without treating another incomplete
   moving revision as final acceptance.
3. **Publish the supported contract and results.** Reconcile the existing design
   documentation and issues #233/#234 with actual supported edits, restart
   boundaries, extension APIs and measured baselines. Do not claim untested paths.

The Windows fixes are integrated. The full 21-command Windows acceptance run
applies to an earlier revision; focused Windows checks followed the fixes, without
a full acceptance run of newer code. See [the Windows report](hot-reload-windows-verification.md).

The application-state callbacks and rapid-reload correction passed 97 engine
tests and five catalog integration tests on both macOS and Linux. Windows has not
tested that revision.

The subsequent compatibility metadata change passed 27 build tests and 99 engine
tests on macOS and Linux, and six catalog reload integration tests on macOS.
The latter include actual generated-manifest admission for binding-path edits and
application state restoration. Final native acceptance remains a separate gate.

The inline-resource implementation passed 106 engine tests on macOS and Linux.
It covers component/panel dictionary identity, application keys/children, removed
keys, repeated commit/rollback, deferred resource identity and ownership transfer.
A root inline-theme test builds its template before commit, rejects missing or
invalid template resources, and verifies that a later successful template resolves
against the preserved root dictionary. Markup tests additionally check weak context
rebasing, nested scopes on unwind, and deferred setter errors without activating
data bindings: 271 markup and 205 XAML tests pass on both macOS and Linux.
The engine suite also reproduces and fixes a realized brush losing its
DynamicResource anchor when the candidate root dies; repeated rollback/commit,
subsequent resource changes and retired-brush/root release are covered.
All seven ControlCatalog reload integration tests pass on macOS, including
resource replacement/removal through the registered component adapter with the
catalog's assembly identity and assets. Commands:

```sh
cargo test --locked -p ferroui-hot-reload --lib
cargo test --locked -p ferroui-markup -p ferroui-markup-xaml --features ferroui-markup-xaml/hot-reload --lib
cargo test --locked -p control-catalog --features hot-reload --lib tests::hot_reload
```

The Linux release-mode check also passes with development features requested:

```sh
cargo check --locked -p control-catalog -p ferroui-hot-reload -p ferroui-hot-patch --features control-catalog/hot-reload --release
```

The inline-style increment passed 112 engine tests on macOS and Linux and eight
ControlCatalog reload integration tests on macOS, using the engine/catalog commands
above. Coverage includes repeated commit/rollback, application style ordering,
resource precedence, retired style/template release, and rejecting source-writing
setter bindings before preparation changes the model. A reproduced regression
where a style-selected theme bypassed template validation is covered: invalid
resources fail before commit, and a corrected template applies after layout.
The catalog test exercises inline styles through its actual component adapter.
The Linux release check above also passes for this increment with the development
feature requested; active reload modules remain gated out by `debug_assertions`.

These checks do not close the remaining handler, mixed-dictionary, template-state,
or final native acceptance requirements.

## Follow-ups outside the first-release requirement

- Native filesystem notification APIs: directory polling already implements the
  requested watching behavior; atomic saves and debounce correctness remain required.
- Registering every ControlCatalog page rather than representative integrations.
- Generalizing the dedicated sample supervisor to arbitrary application layouts.
- Restoring state for ambiguous repeated-template identities: the plan requires
  compatible, unambiguous matches, so deliberately skipping ambiguous matches is valid.
- Patching arbitrary closures, async state machines or compositor callbacks.
- Visual designer tooling, explicitly deferred in the original plan.
- Broader statistical latency distributions beyond the required separate measured
  XAML and Rust baselines.

These follow-ups must not keep the original goal open after its required work passes.
