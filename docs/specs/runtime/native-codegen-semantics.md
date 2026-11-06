# Native component generation semantics

Status: current contract, auto-ui/PLAN-060.

For a supported typed native array whose element implements Clone and Default,
an indexed read uses the generated `__at_idx` helper. A negative index or an
index outside the array returns the element's Default; an in-range read returns
the selected element. Empty arrays follow the same rule. The returned owned
element can participate in a subsequent field projection. Existing JSON Value
array access keeps its established `at_owned`/`get_owned` path.

This rule does not add language types or promise that arbitrary Rust types are
representable by the generator. The generated program must still compile for
its selected element type; a successful build of Calendar is one consumer
case, not proof of every possible type or application.

Generated native route components retain the existing per-app store ownership
described in [component and store instances](component-and-store-instances.md).
Page Init uses the owning root's child-message wrapper so its store updates
survive the next view. A root with no local timer forwards only the active
outlet page's existing single timer and wraps its message. Local timer
precedence and the first-timer-per-component boundary remain unchanged.

Generated frontend freshness includes the tool's front-layout version. Runtime
SDK changes still require Cargo dependency checking and a rebuilt application;
an old published executable is not evidence for a changed SDK. Consumer receipts
bind the selected roots/tool, generated inputs, workspace lock and target root,
and must match published and Cargo executable hashes. A failed generation or
build cannot be replaced by a successful stale executable.

Verification includes generator index/route tests, tools' generated merged
backend regressions, the six native application build gate, and the actual
Calendar/Notes/Todo/Blog consumer receipts. Their individual scope and source
revisions remain part of the evidence.
