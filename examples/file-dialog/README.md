# FileDialog native fixture

Run from this directory with the matching AutoUI tool:

```sh
auto run -r vm
```

`AUTO_UI_TOOL_BIN` selects the exact built `auto-ui-tool`; `AUTOUI_MCP_PORT`
selects its isolated MCP endpoint. `FILE_DIALOG_INITIAL_DIRECTORY` supplies
the opening directory (a complete file path prefills the save-mode basename).
`FILE_DIALOG_DEFAULT_NAME` / `FILE_DIALOG_DEFAULT_EXT` seed the save-mode
`defaultFileName` / `defaultExtension` props. `FILE_DIALOG_RESULT_PATH` names
a durable JSON result file written only by the ordinary accepted handler. It
contains `{"files":[{"name":"…","path":"…"}]}`.

The fixture offers all four PLAN-084 modes (选择文件 / 单选文件 / 选择目录 /
保存文件), an observable background action, a theme switch, accepted/cancel
counters and the public error event. The selector never writes fixture bytes:
save acceptance returns the resolved target path without creating it. Use
“退出夹具” after closing the dialog for a documented successful process exit.
The fixture owns no backend or library database. Its durable output proves only
the component callback; Reader data acceptance is verified in Reader.

Run the product gate with `python tests/product/verify_file_dialog.py --help`
from the AutoUI repository. It creates isolated input/output fixtures, discovers
live controls, tests negative controls, then closes and reopens its own session.

The gate pins source discovery to the repository's workspace siblings
(auto-lang/ui/man/web/down), and records those roots with binary hashes. It does
not fall back to other moving checkouts after copying the fixture outside the
workspace. Normal quit waits up to five seconds for all captured child processes
to disappear before testing the closed endpoint and starting a fresh session;
forced failure cleanup never counts as a successful quit.
