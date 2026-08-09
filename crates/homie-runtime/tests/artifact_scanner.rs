use homie_runtime::{ArtifactKind, scan_artifacts};

#[test]
fn scan_artifacts_detects_pr_preview_link_and_ports() {
    let scan = scan_artifacts(
        "opened https://github.example/repo/pull/42 preview http://localhost:3000 docs https://example.invalid/page",
    );
    assert_eq!(scan.ports.len(), 1);
    assert_eq!(scan.ports[0].port, 3000);
    assert!(
        scan.artifacts
            .iter()
            .any(|artifact| artifact.kind == ArtifactKind::PullRequest
                && artifact.url == "https://github.example/repo/pull/42")
    );
    assert!(
        scan.artifacts
            .iter()
            .any(|artifact| artifact.kind == ArtifactKind::Preview
                && artifact.url == "http://localhost:3000")
    );
    assert!(
        scan.artifacts
            .iter()
            .any(|artifact| artifact.kind == ArtifactKind::Link
                && artifact.url == "https://example.invalid/page")
    );
}

#[test]
fn scan_artifacts_deduplicates_urls_and_ports() {
    let scan = scan_artifacts("http://localhost:3000 http://localhost:3000");
    assert_eq!(scan.ports.len(), 1);
    assert_eq!(scan.artifacts.len(), 1);
}
