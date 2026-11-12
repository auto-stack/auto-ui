# FileDialog native fixture

Run from this directory with the matching AutoUI tool:

```sh
auto run -r vm
```

`AUTO_UI_TOOL_BIN` selects the exact built `auto-ui-tool`; `AUTOUI_MCP_PORT`
selects its isolated MCP endpoint. `FILE_DIALOG_INITIAL_DIRECTORY` supplies
the opening directory. `FILE_DIALOG_RESULT_PATH` names a durable JSON result
file written only by the ordinary accepted handler. It contains
`{"files":[{"name":"…","path":"…"}]}`.

The fixture offers multiple/single selection, an observable background action,
a theme switch, accepted/cancel counters and the public error event. Use
“退出夹具” after closing the dialog for a documented successful process exit.
The fixture owns no backend or library database. Its durable output proves only
the component callback; Reader data acceptance is verified in Reader.

Run the product gate with `python tests/product/verify_file_dialog.py --help`
from the AutoUI repository. It creates isolated input/output fixtures, discovers
live controls, tests negative controls, then closes and reopens its own session.
