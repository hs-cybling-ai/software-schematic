# Native/web bridge protocol v1

Every message is a JSON object with `version`, `type`, nullable `requestId`, nullable `format`, and an object `payload`. Native commands are `load`, `requestExport`, `undo`, `redo`, and `appearance`. Web events are `ready`, `changed`, `exported`, `warnings`, and `failed`.

`load` and `requestExport` require unique request identifiers. XML exists only in the payload and is passed as data to `WKWebView.callAsyncJavaScript`; it is never concatenated into source code. The native side serializes export requests per tab and correlates `exported` or `failed` replies by identifier. Unknown versions, formats, and commands are rejected.
