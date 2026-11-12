# Cua Driver MCP tool output format

Every call keeps the standard MCP `ToolResult` envelope:

- `content` contains human-readable text and optional images;
- `structuredContent` contains the machine-readable successful result;
- `isError` distinguishes tool failure from a successful outcome.

Text is diagnostic, not a stable parsing API.

## Action tools

Successful pointer, keyboard, value, and browser-input actions return the
closed shared `ActionResult` in `structuredContent`:

```json
{
  "effect": "unverifiable",
  "route": "global_input",
  "delivery": {"mode": "foreground"},
  "escalation": {"target": "page", "reason": "effect_unconfirmed"}
}
```

Do not parse platform route names, coordinates, targets, or the removed
`verified` bit from text. Use `effect`, `route`, `delivery`, `evidence`, and
`escalation`, then call `verify_state` or take a fresh snapshot for the task
postcondition. See [Action results and postcondition
verification](action-result-contract.md).

## Observation and state tools

Observation tools retain their typed tool-specific structured payloads.
`get_window_state`, for example, returns the accessibility outline and element
records in `structuredContent` and can attach a PNG as image content. A
multimodal harness interprets the image; Cua Driver does not OCR it or assign
task meaning.

On Windows, `get_window_state.elements_complete` is true only when the
unprojected UI Automation walk visited every exposed node without a traversal
bound, timeout, or enumeration failure skipping a node or subtree. Reaching
`max_elements` or `max_depth` exactly at the end of a tree remains complete.
`query` projects the returned Markdown and structured element rows, but does
not make the underlying snapshot incomplete; compare `total_element_count`
with `returned_element_count` to measure that projection. Partial MSAA trees,
unusable UIA trees, and process-root fallbacks remain incomplete because they
cannot prove the exact target window's full UIA tree.
Internal `verify_state` reads also remain incomplete while the Windows
structured projection omits display-only UIA nodes. A complete native walk
does not make that projection exhaustive.

On macOS, ordinary `get_window_state` reads report `elements_complete: false`:
they collapse layout containers, drop unlabelled rows and fold text rows into
their parent, so a missing row proves nothing. The internal `verify_state`
read walks without that compaction and reports `elements_complete: true` when
the walk of the exact requested window finished without a node or time
budget stop and without passing over anything: no children below
`max_depth`, no failed `AXChildren` read, no node whose attributes could not
be read, no omitted tabs. Only then can `verify_state` answer `unsatisfied`
for an element that is not there instead of `unknown`.

Answers that name a pid also report `process_instance`: an opaque label of the
process that owned the pid when the answer was read (its kernel start time:
macOS `pbi_start_tvsec`/`pbi_start_tvusec`, Linux `/proc/<pid>/stat` field 22,
Windows `GetProcessTimes` creation time), on each `list_apps` running app, each
`list_windows` record and the `get_window_state` answer. Compare it for
equality only; the same pid with a different label is a different process. It
is reported only when the start time read before and after the answer agree,
so a pid that was reused or unreadable mid-read gets none. On macOS
`get_window_state` element tokens are also bound to that process (a
screenshot-only read that refreshed an existing snapshot omits the label,
since that snapshot's tokens keep the instance they were read from): once the pid
names another process start, using them refuses with
`process_instance_mismatch` (`process_instance_unverified` when the identity
could not be read).
