## ADDED Requirements

### Requirement: Complete CLI grammar
The CLI SHALL implement the Diri session, worktree, event, artifact, port and forward operations through `homie-client` with stable human, JSON and NDJSON output.

#### Scenario: Session selector is ambiguous
- **WHEN** title or prefix resolution matches more than one session
- **THEN** the command fails closed and lists safe candidate identifiers

#### Scenario: Event subscription is requested
- **WHEN** a caller subscribes with a cursor and filter
- **THEN** the CLI streams ordered NDJSON events and reports gaps without buffering indefinitely

### Requirement: Exact MCP tool schemas
Every advertised MCP tool MUST have an exact JSON Schema, executable handler, permission rule and stable result/error contract.

#### Scenario: Tools are listed
- **WHEN** MCP returns `tools/list`
- **THEN** only currently executable tools are present and each tool declares required fields and unknown-field behavior

#### Scenario: Unknown tool is called
- **WHEN** the requested tool is absent from the active catalog
- **THEN** MCP returns JSON-RPC method-not-found rather than a generic execution error

### Requirement: Complete lineage tools
MCP SHALL implement direct and recursive lineage semantics for whoami, list/wait/summarize children, report-to-parent, send and release.

#### Scenario: Session reports to parent
- **WHEN** a bound child calls `report_to_parent`
- **THEN** the parent receives a safe provenance-linked context event

#### Scenario: Session targets an unrelated session
- **WHEN** send, summarize or release lacks lineage/permission authority
- **THEN** the operation is rejected before runtime mutation

### Requirement: Managed browser and test automation
Browser and test tools SHALL execute through a bounded sidecar/runner and SHALL be advertised only when that engine is ready.

#### Scenario: Sidecar is unavailable
- **WHEN** MCP capability discovery runs
- **THEN** browser/test tools are absent or explicitly capability-disabled and no static success is returned

#### Scenario: Sidecar produces an image
- **WHEN** a browser action captures a screenshot
- **THEN** the result contains a safe artifact reference/path rather than inline sensitive bytes
