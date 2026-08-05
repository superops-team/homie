# Pre-Commit Security Baseline

Homie keeps a repository-level pre-commit hook in `.githooks/pre-commit`.

Enable it once per clone:

```bash
git config core.hooksPath .githooks
```

The hook scans staged file names and staged file contents before a commit. It blocks common high-risk cases:

- `.env`, `.netrc`, `.npmrc`, Cargo credentials, local provider config, and private key files;
- PEM/private key blocks;
- GitHub tokens;
- AWS access keys;
- OpenAI-style `sk-*` keys;
- Anthropic `sk-ant-*` keys;
- Slack tokens;
- Google API keys;
- generic `api_key`, `secret`, `token`, `password`, `authorization`, and `bearer` assignments with non-placeholder values.

Commit sanitized templates instead:

- `.env.example`
- `.env.sample`
- `*.example.toml`
- `*.sample.yaml`

Templates must contain placeholders such as `<YOUR_API_KEY>`, `example-token`, or `changeme`, not real credentials.

This hook is a local safety net, not a substitute for GitHub secret scanning or provider-side key rotation. If a real credential is committed, rotate it immediately even if the commit is later removed.
