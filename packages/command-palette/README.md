# CommandPalette

`CommandPalette` presents a modal search field and an ordered list of commands. The caller controls when it opens, supplies command records, and owns permissions and execution. Shortcut strings are display hints.

Add the local package to the consumer manifest and import it relative to the widget source.

```text
dep command_palette { path: "../../packages/command-palette" }
use { package: command_palette from "../../../../packages/command-palette" }
```

The [settings example](../../examples/command-palette/README.md) shows the complete bindings.

## Data and events

Supply command records with a unique nonempty `id`, a `label`, optional group metadata, `keywords`, `shortcut`, `disabled`, and `icon`. Use empty strings or lists for unused fields. Commands can share a label. Local search folds ASCII case, trims surrounding ASCII whitespace, and matches the label, ID and keywords. Provided search preserves application order and renders headings at group boundaries.

Pass `open`, `revision`, `commands` and optional `title`, `placeholder`, `query`, `wrapNavigation`, `filterMode`, `response` and `errorMessage`. In provided mode, set `resultsSession`, `resultsQuerySeq` and `resultsQuery` together with `commands` and increment `revision`.

- `queryChanged(str)` sends JSON with `instanceToken`, `session`, `querySeq`, and `query`.
- `invokeRequested(str)` adds `seq`, `commandId` and `baseRevision` to the query envelope. Validate the active session and authorization before acting.
- `closeRequested(str)` sends `instanceToken`, `session` and `reason` (`escape` or `cancel`). Close by setting `open` to false.
- `error(str)` sends `code` and `message` for invalid configuration.

To acknowledge a request, pass JSON as `response` with the original `instanceToken`, `session`, `seq` and `commandId`, plus `status` and `message`. An accepted response holds the pending lock until the caller closes the palette. A rejected response displays the message and enables retry.

The default slot can supply application controls such as the Launcher category bar. It is available while the palette accepts user input. Up and Down move among enabled results, Enter runs the current result, and Escape requests dismissal. Tab follows standard modal focus order.
