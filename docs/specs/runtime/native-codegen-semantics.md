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

Native `Http.post`, `Http.get` and `Http.get_json` use the backend profile
selected for that generated app. Root-relative paths resolve against its baked
base; absolute URLs retain their origin. The resolver does not modify the host
environment. The generated base and the `profile/http-base` receipt input come
from the same selected configuration; changing it invalidates artifact reuse.
Applications using this HTTP facade still require their real backend service;
API entity structs alone do not embed that service.

The facade preserves JSON response bodies, including API error responses.
Transport failure is reported through the existing UI catch path with a static
diagnostic that contains no request URL, token or body. Declared String state
targets retain JSON string identities rather than applying the field-name
integer heuristic. Integer state reads convert the original JSON value once;
already composed string expressions retain their existing string path.

Native child input handlers retain calls to declared `msg` callback props.
Emissions belong to that child instance and are consumed in call order after
child dispatch and store writeback, through the parent's declared message
binding. Parameters are checked against that message's payload types and
arity; failures use static diagnostics. Unbound optional callbacks are consumed
without dispatch. Subsequent views and inputs do not replay consumed events.
The existing per-component child-instance model remains: conflicting bindings
for one child type and bindings with fixed arguments are rejected during
generated compilation. This rule covers normal child message dispatch, not a
new callback protocol for constructor execution.

String concatenation of known JSON reads converts the original value with the
existing string shim. Field-name integer guesses cannot turn an opaque intent
key into zero. Pushing a known JSON read into a dynamic Value collection retains
the original object/array/scalar shape. Typed string/record and numeric contexts
keep their declared types, and indexed JSON reads retain safe array indexing.

Native frontend receipts also cover the content and file set of the selected
local backend source directory: `src/back` takes precedence over `back`, as in
the local API parser. Backend-only edits, additions and removals require code
regeneration; backend input read failures stop receipt publication. This local
input rule does not extend the separate external backend-project resolver.

### 普通布局样式的滚动转换（PLAN-060 r6）

原生Rust生成的普通row/col在求值实际样式后，将overflow-y-auto/overflow-auto转换为真正的Y轴Scrollable。动态样式同样只求值一次，转换不重放组件消息。无overflow、overflow-hidden与显式scroll保持原有结构；显式scroll不重复包装。背景、边框、圆角等视口视觉与内容布局的拆分复用VM的同一实现，原有列/行子项、间距、padding和点击回调保留。该规则不声称其他尚未接入的标签支持，也不新增自动滚到底或横向滚动合同。生成身份front-layout/v13使旧输出在正式构建时重生成；所建、所发布、所测的输入与产物仍按现有receipt验证。
