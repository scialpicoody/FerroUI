# Hot reload completion audit

Audited on 2026-10-20. The goal remains
active. This checklist fixes the completion scope; historical validation notes
retain their original source revisions.

## Implemented, with recorded verification

| Requirement | Current evidence and limits |
| --- | --- |
| Runtime XAML and last-good replacement | Session, overlay, detached preparation, reversible content/component/panel adapters and failure tests exist. Native catalog probes ran on all three platforms at recorded revisions. |
| Sessions and build/loading integration | Manifest, persistent/scoped populate hooks, weak targets, document buffers, directory polling/debounce, superseded revisions, transformed-document cache and debug/release gates exist. Handler manifest coverage remains incomplete below. |
| Stable windows, roots and view models | Native identity probes and lifecycle tests cover supported adapters. Root literal/static-resource/binding ownership supports removal and rollback. |
| Compatible UI state | Focus, text/caret/selection, scroll and selection-model tests exist. Current working changes add application state callbacks and prevent two rapid accepted revisions from losing queued state. |
| Cached controls and subscriptions | Transactional refresh hooks and the Expander sample exercise rollback, disposal and retired-tree release. Ten catalog pages are registered. This is representative integration, not all-page support. |
| Included resources/styles and static consumers | Registered application containers discover direct live includes; dependency revisions prepare resources and consumers coherently. Resource/theme probes and include lifetime tests exist. Standard inline component ownership remains incomplete below. |
| Explicit Rust patching | Real Subsecond patches update existing callback subscriptions, with stable framework identity, conservative admission and controlled application. Native patch evidence exists for macOS, Linux and Windows at recorded revisions. |
| Supervisor and restart | The dedicated sample owns admission/build/application, retains the old process on failed replacement builds and restores versioned application/window/navigation state. Native supervisor evidence exists. |
| Combined updates and measurements | Combined XAML/Rust acceptance and stage/save-to-render measurements exist for a small fixture and representative catalog pages. These are functional baselines, not statistical performance guarantees. |

## Required completion work

1. **Close standard component integration gaps.** The shared component/panel
   preparation currently rejects all XAML event handlers and most root property
   elements, including inline resources/styles. Implement the needed ownership
   adapters and exercise standard handlers and resource/style replacement through
   actual component-loading paths. Preserve the documented restart boundary for
   unsupported custom mutations and irreversible user code.
2. **Finish compatibility metadata.** Build-manifest handler signatures and
   conservative unknown-type handling still need completion. Keep class, named
   control, handler and compiled-binding contracts consistent between generated
   and runtime loading.
3. **Finalize the current state change.** The working tree's application-state
   callbacks and rapid-reload correction passed 97 engine tests and five catalog
   integration tests on both macOS and Linux. Commit the reviewed implementation
   and retain its source-specific evidence; Windows has not tested these changes.
4. **Freeze a final implementation revision and close acceptance.** Reuse existing
   evidence where behavior and inputs are unchanged, but run the final relevant
   native XAML, Rust, combined, restart, lifecycle and release-boundary gates on
   macOS, Windows and Linux. Fix failures without treating another incomplete
   moving revision as final acceptance.
5. **Publish the supported contract and results.** Reconcile the existing design
   documentation and issues #233/#234 with actual supported edits, restart
   boundaries, extension APIs and measured baselines. Do not claim untested paths.

The Windows fixes are integrated. The full 21-command Windows acceptance run
applies to an earlier revision; focused Windows checks followed the fixes, without
a full acceptance run of newer code. See [the Windows report](hot-reload-windows-verification.md).

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
