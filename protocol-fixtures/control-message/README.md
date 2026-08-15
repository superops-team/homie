# Control Message Fixtures

These fixtures define the current Homie control-channel envelope shared by
Swift `HomieProtocol.ControlMessage` and Rust `homie-proto::ControlMessage`.

The envelope is intentionally not tagged. Decode precedence is:

1. request when `method` is present and non-null;
2. event when `event` is present and non-null;
3. failure response when `err` is present and non-null;
4. success response otherwise, using `ok` or `null` when `ok` is absent.

Rules:

- Fixtures are test contracts only. They are not runtime configuration.
- Fixtures must not be copied into app bundles.
- Use synthetic payloads only.
- Do not include prompts, Authorization headers, cookies, provider tokens,
  private keys, local private paths, or user session content.
