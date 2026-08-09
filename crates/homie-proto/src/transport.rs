use serde::{Deserialize, Serialize};

use crate::stream::StreamKind;

pub const WIRE_MAJOR: u16 = 1;
pub const WIRE_MINOR: u16 = 0;
pub const PREFACE_LEN: usize = 12;
pub const FRAME_HEADER_LEN: usize = 24;
pub const MAX_FRAME_LEN: usize = 16 * 1024 * 1024;
pub const MAX_CONTROL_PAYLOAD: usize = 4 * 1024 * 1024;
pub const MAX_OUTPUT_PAYLOAD: usize = 64 * 1024;

const PREFACE_MAGIC: &[u8; 8] = b"HOMIEIPC";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Preface {
    pub major: u16,
    pub minor: u16,
}

impl Preface {
    #[must_use]
    pub fn encode(self) -> [u8; PREFACE_LEN] {
        let mut encoded = [0_u8; PREFACE_LEN];
        encoded[..PREFACE_MAGIC.len()].copy_from_slice(PREFACE_MAGIC);
        encoded[8..10].copy_from_slice(&self.major.to_be_bytes());
        encoded[10..12].copy_from_slice(&self.minor.to_be_bytes());
        encoded
    }

    pub fn decode(encoded: &[u8]) -> Result<Self, TransportError> {
        if encoded.len() < PREFACE_LEN {
            return Err(TransportError::IncompletePreface {
                actual: encoded.len(),
            });
        }
        if &encoded[..PREFACE_MAGIC.len()] != PREFACE_MAGIC {
            return Err(TransportError::InvalidPrefaceMagic);
        }
        let preface = Self {
            major: u16::from_be_bytes([encoded[8], encoded[9]]),
            minor: u16::from_be_bytes([encoded[10], encoded[11]]),
        };
        if preface.major != WIRE_MAJOR {
            return Err(TransportError::UnsupportedPrefaceMajor(preface.major));
        }
        Ok(preface)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointRole {
    Client,
    Server,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum FrameKind {
    Hello = 1,
    HelloAck = 2,
    Request = 3,
    Response = 4,
    Event = 5,
    StreamOpen = 6,
    StreamOpened = 7,
    StreamReset = 8,
    StreamClose = 9,
    Output = 16,
    Input = 17,
    Resize = 18,
    Grid = 19,
    Modes = 20,
    ReplayBegin = 21,
    ReplayEnd = 22,
    Ping = 23,
    Pong = 24,
}

impl TryFrom<u8> for FrameKind {
    type Error = TransportError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Hello),
            2 => Ok(Self::HelloAck),
            3 => Ok(Self::Request),
            4 => Ok(Self::Response),
            5 => Ok(Self::Event),
            6 => Ok(Self::StreamOpen),
            7 => Ok(Self::StreamOpened),
            8 => Ok(Self::StreamReset),
            9 => Ok(Self::StreamClose),
            16 => Ok(Self::Output),
            17 => Ok(Self::Input),
            18 => Ok(Self::Resize),
            19 => Ok(Self::Grid),
            20 => Ok(Self::Modes),
            21 => Ok(Self::ReplayBegin),
            22 => Ok(Self::ReplayEnd),
            23 => Ok(Self::Ping),
            24 => Ok(Self::Pong),
            _ => Err(TransportError::UnknownFrameKind(value)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameHeader {
    pub version: u16,
    pub kind: FrameKind,
    pub flags: u8,
    pub stream_id: u32,
    pub message_id: u64,
    pub sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Frame {
    pub header: FrameHeader,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn encode(&self, sender: EndpointRole) -> Result<Vec<u8>, TransportError> {
        self.validate(sender)?;
        let frame_len = FRAME_HEADER_LEN
            .checked_add(self.payload.len())
            .ok_or(TransportError::FrameLengthOverflow)?;
        if frame_len > MAX_FRAME_LEN {
            return Err(TransportError::FrameTooLarge(frame_len));
        }
        let frame_len =
            u32::try_from(frame_len).map_err(|_| TransportError::FrameLengthOverflow)?;
        let mut encoded = Vec::with_capacity(4 + frame_len as usize);
        encoded.extend_from_slice(&frame_len.to_be_bytes());
        encoded.extend_from_slice(&self.header.version.to_be_bytes());
        encoded.push(self.header.kind as u8);
        encoded.push(self.header.flags);
        encoded.extend_from_slice(&self.header.stream_id.to_be_bytes());
        encoded.extend_from_slice(&self.header.message_id.to_be_bytes());
        encoded.extend_from_slice(&self.header.sequence.to_be_bytes());
        encoded.extend_from_slice(&self.payload);
        Ok(encoded)
    }

    pub fn decode(
        encoded: &[u8],
        sender: EndpointRole,
    ) -> Result<Option<(Self, usize)>, TransportError> {
        if encoded.len() < 4 {
            return Ok(None);
        }
        let frame_len =
            u32::from_be_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]) as usize;
        if frame_len < FRAME_HEADER_LEN {
            return Err(TransportError::InvalidFrameLength(frame_len));
        }
        if frame_len > MAX_FRAME_LEN {
            return Err(TransportError::FrameTooLarge(frame_len));
        }
        let encoded_len = 4_usize
            .checked_add(frame_len)
            .ok_or(TransportError::FrameLengthOverflow)?;
        if encoded.len() < encoded_len {
            return Ok(None);
        }

        let mut message_id = [0_u8; 8];
        message_id.copy_from_slice(&encoded[12..20]);
        let mut sequence = [0_u8; 8];
        sequence.copy_from_slice(&encoded[20..28]);
        let header = FrameHeader {
            version: u16::from_be_bytes([encoded[4], encoded[5]]),
            kind: FrameKind::try_from(encoded[6])?,
            flags: encoded[7],
            stream_id: u32::from_be_bytes([encoded[8], encoded[9], encoded[10], encoded[11]]),
            message_id: u64::from_be_bytes(message_id),
            sequence: u64::from_be_bytes(sequence),
        };
        let payload = &encoded[4 + FRAME_HEADER_LEN..encoded_len];
        validate_header(&header, payload.len(), sender)?;
        validate_payload(header.kind, payload)?;
        Ok(Some((
            Self {
                header,
                payload: payload.to_vec(),
            },
            encoded_len,
        )))
    }

    fn validate(&self, sender: EndpointRole) -> Result<(), TransportError> {
        validate_header(&self.header, self.payload.len(), sender)?;
        validate_payload(self.header.kind, &self.payload)
    }
}

fn validate_header(
    header: &FrameHeader,
    payload_len: usize,
    sender: EndpointRole,
) -> Result<(), TransportError> {
    if header.version != WIRE_MAJOR {
        return Err(TransportError::UnsupportedFrameVersion(header.version));
    }
    if header.flags != 0 {
        return Err(TransportError::UnsupportedFlags(header.flags));
    }

    let requires_control_stream = matches!(
        header.kind,
        FrameKind::Hello | FrameKind::HelloAck | FrameKind::Request | FrameKind::Response
    );
    let requires_data_stream =
        !requires_control_stream && !matches!(header.kind, FrameKind::Ping | FrameKind::Pong);
    if (requires_control_stream && header.stream_id != 0)
        || (requires_data_stream && header.stream_id == 0)
    {
        return Err(TransportError::InvalidStreamId {
            kind: header.kind,
            stream_id: header.stream_id,
        });
    }
    if header.kind == FrameKind::StreamOpen {
        let valid_owner = match sender {
            EndpointRole::Client => !header.stream_id.is_multiple_of(2),
            EndpointRole::Server => header.stream_id.is_multiple_of(2),
        };
        if !valid_owner {
            return Err(TransportError::InvalidStreamId {
                kind: header.kind,
                stream_id: header.stream_id,
            });
        }
    }
    if matches!(header.kind, FrameKind::Request | FrameKind::Response) && header.message_id == 0 {
        return Err(TransportError::ZeroMessageId(header.kind));
    }
    if header.kind.is_json() && payload_len > MAX_CONTROL_PAYLOAD {
        return Err(TransportError::ControlPayloadTooLarge(payload_len));
    }
    if header.kind == FrameKind::Output && payload_len > MAX_OUTPUT_PAYLOAD {
        return Err(TransportError::OutputPayloadTooLarge(payload_len));
    }
    Ok(())
}

fn validate_payload(kind: FrameKind, payload: &[u8]) -> Result<(), TransportError> {
    if kind.is_json() {
        if kind == FrameKind::StreamClose && payload.is_empty() {
            return Ok(());
        }
        let mut deserializer = serde_json::Deserializer::from_slice(payload);
        serde::de::IgnoredAny::deserialize(&mut deserializer)
            .map_err(|_| TransportError::InvalidPayload(kind))?;
        deserializer
            .end()
            .map_err(|_| TransportError::InvalidPayload(kind))?;
        return Ok(());
    }

    let valid = match kind {
        FrameKind::Output => payload.len() >= 8,
        FrameKind::Resize => payload.len() == 4,
        FrameKind::ReplayBegin | FrameKind::ReplayEnd => payload.len() == 8,
        FrameKind::Ping | FrameKind::Pong => payload.is_empty(),
        _ => true,
    };
    if !valid {
        return Err(TransportError::InvalidPayload(kind));
    }
    Ok(())
}

impl FrameKind {
    #[must_use]
    const fn is_json(self) -> bool {
        matches!(
            self,
            Self::Hello
                | Self::HelloAck
                | Self::Request
                | Self::Response
                | Self::Event
                | Self::StreamOpen
                | Self::StreamOpened
                | Self::StreamReset
                | Self::StreamClose
        )
    }
}

#[derive(Debug)]
pub struct FrameDecoder {
    sender: EndpointRole,
    buffer: Vec<u8>,
    preface: Option<Preface>,
}

impl FrameDecoder {
    #[must_use]
    pub fn new(sender: EndpointRole) -> Self {
        Self {
            sender,
            buffer: Vec::new(),
            preface: None,
        }
    }

    #[must_use]
    pub fn preface(&self) -> Option<Preface> {
        self.preface
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<Frame>, TransportError> {
        let mut frames = Vec::new();
        let mut remaining = bytes;
        while !remaining.is_empty() {
            if self.preface.is_none() {
                let needed = PREFACE_LEN - self.buffer.len();
                let copied = needed.min(remaining.len());
                self.buffer.extend_from_slice(&remaining[..copied]);
                remaining = &remaining[copied..];
                if self.buffer.len() < PREFACE_LEN {
                    break;
                }
                self.preface = Some(Preface::decode(&self.buffer)?);
                self.buffer.clear();
                continue;
            }

            if self.buffer.len() < size_of::<u32>() {
                let needed = size_of::<u32>() - self.buffer.len();
                let copied = needed.min(remaining.len());
                self.buffer.extend_from_slice(&remaining[..copied]);
                remaining = &remaining[copied..];
                if self.buffer.len() < size_of::<u32>() {
                    break;
                }
            }

            let frame_len = u32::from_be_bytes(
                self.buffer[..size_of::<u32>()]
                    .try_into()
                    .map_err(|_| TransportError::FrameLengthOverflow)?,
            ) as usize;
            if frame_len < FRAME_HEADER_LEN {
                return Err(TransportError::InvalidFrameLength(frame_len));
            }
            if frame_len > MAX_FRAME_LEN {
                return Err(TransportError::FrameTooLarge(frame_len));
            }
            let encoded_len = size_of::<u32>()
                .checked_add(frame_len)
                .ok_or(TransportError::FrameLengthOverflow)?;
            let needed = encoded_len - self.buffer.len();
            let copied = needed.min(remaining.len());
            self.buffer.extend_from_slice(&remaining[..copied]);
            remaining = &remaining[copied..];
            if self.buffer.len() < encoded_len {
                break;
            }

            let Some((frame, consumed)) = Frame::decode(&self.buffer, self.sender)? else {
                return Err(TransportError::FrameLengthOverflow);
            };
            frames.push(frame);
            debug_assert_eq!(consumed, self.buffer.len());
            self.buffer.clear();
        }
        Ok(frames)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_decoder_rejects_oversized_length_before_copying_trailing_chunk() {
        let mut decoder = FrameDecoder::new(EndpointRole::Client);
        decoder
            .push(
                &Preface {
                    major: WIRE_MAJOR,
                    minor: WIRE_MINOR,
                }
                .encode(),
            )
            .expect("decode preface");
        let declared_len = u32::try_from(MAX_FRAME_LEN + 1).expect("frame limit fits u32");
        let mut hostile_chunk = declared_len.to_be_bytes().to_vec();
        hostile_chunk.resize(1024 * 1024, 0);

        let error = decoder
            .push(&hostile_chunk)
            .expect_err("oversized frame must fail");

        assert_eq!(error, TransportError::FrameTooLarge(MAX_FRAME_LEN + 1));
        assert_eq!(decoder.buffer, declared_len.to_be_bytes());
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TransportError {
    #[error("incomplete preface: received {actual} of {PREFACE_LEN} bytes")]
    IncompletePreface { actual: usize },
    #[error("invalid preface magic")]
    InvalidPrefaceMagic,
    #[error("unsupported preface major version {0}")]
    UnsupportedPrefaceMajor(u16),
    #[error("frame length {0} is smaller than the fixed header")]
    InvalidFrameLength(usize),
    #[error("frame length {0} exceeds the configured limit")]
    FrameTooLarge(usize),
    #[error("frame length cannot be represented on the wire")]
    FrameLengthOverflow,
    #[error("unknown frame kind {0}")]
    UnknownFrameKind(u8),
    #[error("unsupported frame version {0}")]
    UnsupportedFrameVersion(u16),
    #[error("unsupported frame flags {0}")]
    UnsupportedFlags(u8),
    #[error("invalid stream id {stream_id} for {kind:?}")]
    InvalidStreamId { kind: FrameKind, stream_id: u32 },
    #[error("message id must be non-zero for {0:?}")]
    ZeroMessageId(FrameKind),
    #[error("control payload length {0} exceeds the configured limit")]
    ControlPayloadTooLarge(usize),
    #[error("output payload length {0} exceeds the configured limit")]
    OutputPayloadTooLarge(usize),
    #[error("invalid payload for {0:?}")]
    InvalidPayload(FrameKind),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientRole {
    App,
    Cli,
    Mcp,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StableErrorCode {
    MethodNotFound,
    BadRequest,
    VersionMismatch,
    Unauthorized,
    Unavailable,
    Timeout,
    Backpressure,
    ResyncRequired,
    Internal,
}

impl StableErrorCode {
    pub const ALL: [Self; 9] = [
        Self::MethodNotFound,
        Self::BadRequest,
        Self::VersionMismatch,
        Self::Unauthorized,
        Self::Unavailable,
        Self::Timeout,
        Self::Backpressure,
        Self::ResyncRequired,
        Self::Internal,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MethodNotFound => "method_not_found",
            Self::BadRequest => "bad_request",
            Self::VersionMismatch => "version_mismatch",
            Self::Unauthorized => "unauthorized",
            Self::Unavailable => "unavailable",
            Self::Timeout => "timeout",
            Self::Backpressure => "backpressure",
            Self::ResyncRequired => "resync_required",
            Self::Internal => "internal",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelloRequest {
    pub wire_major: u16,
    pub wire_minor: u16,
    pub client_name: String,
    pub client_version: String,
    pub client_role: ClientRole,
    pub process_id: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelloResponse {
    pub wire_major: u16,
    pub wire_minor: u16,
    pub daemon_build: String,
    pub daemon_version: String,
    pub daemon_pid: u32,
    pub daemon_instance_id: String,
    pub executable_hash: String,
    pub method_capabilities: Vec<String>,
    pub stream_capabilities: Vec<StreamKind>,
    pub event_oldest_seq: u64,
    pub event_latest_seq: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DaemonStatusKind {
    Ready,
    PreparingShutdown,
    ShuttingDown,
    Unhealthy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonStatus {
    pub status: DaemonStatusKind,
    pub daemon_pid: u32,
    pub daemon_instance_id: String,
    pub daemon_version: String,
    pub method_capabilities: Vec<String>,
    pub stream_capabilities: Vec<StreamKind>,
    pub event_oldest_seq: u64,
    pub event_latest_seq: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AckResult {
    pub ok: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShutdownResult {
    pub acknowledged: bool,
}
