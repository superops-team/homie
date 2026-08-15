# Protocol Contract Golden Fixtures Code Review Round 2

## 1. Scope

Second-pass review focused on hidden fixture semantics and future drift risks.

## 2. Hidden Risk Review

| Risk | Result | Evidence / Handling |
|---|---|---|
| Canonical JSON comparison might be order-sensitive | pass: both tests compare decoded JSON values, not raw text ordering |
| Request `params: null` may diverge from absent params | pass: shared fixture covers `request-null-params-canonical-omits` and both languages canonicalize it to omitted params |
| Event missing params may diverge | pass: shared fixture covers `event-without-params-canonical-null` and both languages canonicalize params to null |
| Response missing `ok` may diverge | pass: shared fixture covers `response-ok-absent-canonical-null` |
| Envelope discrimination order may drift | pass: fixtures cover `method-wins-over-event` and `event-wins-over-error` |
| Sensitive scan might self-match README policy text | fixed in functional case: scan targets only `*.json` fixture payloads |
| Fixture files might be packaged accidentally | pass: package/dev scripts do not reference `protocol-fixtures`; plan marks fixtures test-only |

## 3. Not Changed

- No Swift/Rust production protocol code changed.
- No schema/codegen added.
- No CI workflow changed in this slice.
- No app package path copies `protocol-fixtures`.

## 4. Conclusion

No P0/P1 hidden risks remain for this first slice.
