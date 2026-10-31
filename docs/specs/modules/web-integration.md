# Web integration (PLAN-890)

UI owns rendering, compiler/tool/worker/proxy and non-HTTP adapters. HTTP runtime,
service/client generation and HTTP templates have one owner in auto-web; current
Web rules are [canonical there](../../../../auto-web/docs/specs/project.md).

The tool, VM worker and back-proxy consume fixed sibling Web source and compose
its public adapter explicitly. Typed invocation launch crosses worker boundaries;
config/profile/hash/Host policy cannot be replaced by a worker environment.
Core VM/future remains local/!Send and cleanup retains its original owned-child,
port and resource completion obligations. Non-HTTP behavior and public tool/CLI
semantics stay with UI. Rust generation consumes the selected Web source and
shared auto-generation-core mechanisms rather than old mixed HTTP generators.

Vue handler emission keeps lexical snapshot variables and delegated captures
local; only actual shadowed state uses the internal module alias. Original
getters/ordinary output and initializer/block/loop/closure/catch binding rules
remain compatible. Fixed Blog b42a660 inputs were unchanged: Vue15/15 and
native22/22, with explicit cold Rust compile before the runtime profiles.
The UI generator owner module866PASS/two original ignores and four new controls
are scoped evidence, not a claim that the later whole combination ran.

Owner tests stay `cargo t`/`cargo tu`/`cargo ti` with their original conditions,
resource constraints and single-process ui_gen semantics. The language four-owner
gate records actual selected and passed IDs and source/lock/resource/tool
identities. Original libtest ignores are preserved from same-source ui-full flags,
never counted as executed PASS or arbitrary receipt exclusions. Experimental AG
milestone suites are not newly required by the Web merge.

Reviewed implementation cohort: language29cf44480, Webe01232b, UI9aad12c,
Man cb4ab3d, Downc3464d0. Git linearization/docs descendants and actual final
landed cohort are read back in the central receipt; these pins are not relabeled
as newer binary producers. Final gate/landing/recovery/cleanup status:
[central PLAN-890](../../../../auto-lang/docs/plans/archive/890-auto-web-independent-repository-delivery.md).
No remote publication or additional UI structural work is implied.

Each proxy session retains canonical compiled return types separately from
diagnostic Type Display strings. String-list replies use the language-owned
typed converter and that session's declaration, preserving the existing VM list,
iterator and reference-count ABI; integer and unknown returns retain numeric JSON.

## 最终交付记录（2026-10-31）

r7冻结四仓门与发行已通过，实际合入、主干rebase及用户免追加测试范围，统一见语言仓 `docs/plans/archive/890-auto-web-independent-repository-delivery.md`。历史producer pins只标识实际执行来源，不冒称后续主干更新已重测。

> PLAN-107：由 docs/web-integration.md 迁入 canonical Spec；职责及历史证据边界保持，旧路径仅转引。
