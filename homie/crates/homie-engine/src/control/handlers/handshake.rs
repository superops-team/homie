use super::*;

impl ControlServer {
    pub(crate) fn hello(&self, params: Option<JsonValue>) -> Result<JsonValue, ControlError> {
        let proto = params
            .as_ref()
            .and_then(|value| value.get("proto"))
            .and_then(Value::as_u64)
            .unwrap_or(WIRE_VERSION as u64);
        if proto != WIRE_VERSION as u64 {
            return Err(ControlError::version_mismatch(format!(
                "client speaks protocol {proto}, this engine speaks {WIRE_VERSION}"
            )));
        }
        Ok(json!({
            "proto": WIRE_VERSION,
            "build": BUILD,
            "engineKind": homie_proto::RUST_ENGINE_KIND,
            "pid": std::process::id() as i32,
            "executableHash": process_executable_hash(),
        }))
    }

    pub(crate) fn session_capabilities(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionCapabilitiesParams = decode(params)?;
        let registry = self.registry.lock().map_err(poisoned)?;
        let record = registry
            .record(&p.session_id.0)
            .ok_or_else(|| ControlError::not_found(p.session_id.0.clone()))?;
        let capabilities =
            crate::driver::capabilities_for_manifest_id(record.effective_kind().id());
        encode(&homie_proto::SessionCapabilitiesResult {
            session_id: p.session_id,
            capabilities,
        })
    }
}
