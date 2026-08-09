#![allow(dead_code)]

use std::path::{Path, PathBuf};

use homie_proto::stream::StreamKind;
use homie_proto::transport::{
    EndpointRole, FRAME_HEADER_LEN, Frame, FrameHeader, FrameKind, HelloRequest, HelloResponse,
    MAX_FRAME_LEN, PREFACE_LEN, Preface, WIRE_MAJOR, WIRE_MINOR,
};
use homie_proto::{ControlMessage, ErrorEnvelope, RequestId};
use serde_json::Value;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{UnixListener, UnixStream};

pub struct MockSocket {
    _temp: tempfile::TempDir,
    endpoint: PathBuf,
    listener: UnixListener,
}

impl MockSocket {
    pub fn bind() -> Self {
        let temp = tempfile::tempdir().expect("tempdir");
        let endpoint = temp.path().join("daemon.sock");
        let listener = UnixListener::bind(&endpoint).expect("bind mock UDS");
        Self {
            _temp: temp,
            endpoint,
            listener,
        }
    }

    pub fn endpoint(&self) -> &Path {
        &self.endpoint
    }

    pub async fn accept(
        &self,
        methods: &[&str],
        streams: &[StreamKind],
        instance_id: &str,
    ) -> MockPeer {
        let (mut stream, _) = self.listener.accept().await.expect("accept mock client");
        let mut preface = [0_u8; PREFACE_LEN];
        stream.read_exact(&mut preface).await.expect("read preface");
        assert_eq!(
            Preface::decode(&preface).expect("decode preface"),
            Preface {
                major: WIRE_MAJOR,
                minor: WIRE_MINOR
            }
        );
        let hello = read_frame(&mut stream).await;
        assert_eq!(hello.header.kind, FrameKind::Hello);
        let request: HelloRequest = serde_json::from_slice(&hello.payload).expect("hello request");
        assert_eq!(request.wire_major, WIRE_MAJOR);

        let response = HelloResponse {
            wire_major: WIRE_MAJOR,
            wire_minor: WIRE_MINOR,
            daemon_build: "mock-build".to_string(),
            daemon_version: "0.1.0".to_string(),
            daemon_pid: std::process::id(),
            daemon_instance_id: instance_id.to_string(),
            executable_hash: "mock-hash".to_string(),
            method_capabilities: methods.iter().map(|method| (*method).to_string()).collect(),
            stream_capabilities: streams.to_vec(),
            event_oldest_seq: 0,
            event_latest_seq: 0,
        };
        write_frame(
            &mut stream,
            Frame {
                header: FrameHeader {
                    version: WIRE_MAJOR,
                    kind: FrameKind::HelloAck,
                    flags: 0,
                    stream_id: 0,
                    message_id: 0,
                    sequence: 0,
                },
                payload: serde_json::to_vec(&response).expect("encode hello response"),
            },
        )
        .await;
        MockPeer { stream }
    }
}

pub struct MockPeer {
    stream: UnixStream,
}

impl MockPeer {
    pub async fn read_frame(&mut self) -> Frame {
        read_frame(&mut self.stream).await
    }

    pub async fn write_frame(&mut self, frame: Frame) {
        write_frame(&mut self.stream, frame).await;
    }

    pub async fn read_request(&mut self) -> MockRequest {
        let frame = self.read_frame().await;
        assert_eq!(frame.header.kind, FrameKind::Request);
        let message: ControlMessage =
            serde_json::from_slice(&frame.payload).expect("decode request payload");
        let ControlMessage::Request {
            request_id,
            method,
            params,
        } = message
        else {
            panic!("expected request payload");
        };
        assert_eq!(request_id.as_u64(), frame.header.message_id);
        MockRequest {
            message_id: frame.header.message_id,
            method,
            params,
        }
    }

    pub async fn respond_ok(&mut self, message_id: u64, result: Value) {
        let payload = ControlMessage::success(RequestId::from(message_id), result);
        self.write_frame(Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::Response,
                flags: 0,
                stream_id: 0,
                message_id,
                sequence: 0,
            },
            payload: serde_json::to_vec(&payload).expect("encode response"),
        })
        .await;
    }

    pub async fn respond_error(&mut self, message_id: u64, error: ErrorEnvelope) {
        let payload = ControlMessage::failure(RequestId::from(message_id), error);
        self.write_frame(Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::Response,
                flags: 0,
                stream_id: 0,
                message_id,
                sequence: 0,
            },
            payload: serde_json::to_vec(&payload).expect("encode error response"),
        })
        .await;
    }
}

pub struct MockRequest {
    pub message_id: u64,
    pub method: String,
    pub params: Value,
}

async fn read_frame(stream: &mut UnixStream) -> Frame {
    let mut length = [0_u8; 4];
    stream
        .read_exact(&mut length)
        .await
        .expect("read frame length");
    let frame_len = u32::from_be_bytes(length) as usize;
    assert!((FRAME_HEADER_LEN..=MAX_FRAME_LEN).contains(&frame_len));
    let mut encoded = vec![0_u8; 4 + frame_len];
    encoded[..4].copy_from_slice(&length);
    stream
        .read_exact(&mut encoded[4..])
        .await
        .expect("read frame body");
    Frame::decode(&encoded, EndpointRole::Client)
        .expect("decode frame")
        .expect("complete frame")
        .0
}

async fn write_frame(stream: &mut UnixStream, frame: Frame) {
    stream
        .write_all(&frame.encode(EndpointRole::Server).expect("encode frame"))
        .await
        .expect("write frame");
}
