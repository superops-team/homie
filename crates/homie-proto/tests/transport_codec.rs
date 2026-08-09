use homie_proto::transport::{
    EndpointRole, FRAME_HEADER_LEN, Frame, FrameDecoder, FrameHeader, FrameKind,
    MAX_CONTROL_PAYLOAD, MAX_FRAME_LEN, MAX_OUTPUT_PAYLOAD, Preface, WIRE_MAJOR, WIRE_MINOR,
};

fn request_frame(message_id: u64, payload: &[u8]) -> Frame {
    Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Request,
            flags: 0,
            stream_id: 0,
            message_id,
            sequence: 0,
        },
        payload: payload.to_vec(),
    }
}

#[test]
fn frame_round_trip_preserves_exact_preface_and_header_bytes() {
    let preface = Preface {
        major: WIRE_MAJOR,
        minor: WIRE_MINOR,
    };
    let encoded_preface = preface.encode();
    assert_eq!(
        encoded_preface,
        [b'H', b'O', b'M', b'I', b'E', b'I', b'P', b'C', 0, 1, 0, 0]
    );
    assert_eq!(
        Preface::decode(&encoded_preface).expect("decode preface"),
        preface
    );

    let frame = Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Request,
            flags: 0,
            stream_id: 0,
            message_id: 0x0102_0304_0506_0708,
            sequence: 0x1112_1314_1516_1718,
        },
        payload: br#"{}"#.to_vec(),
    };
    let encoded = frame
        .encode(EndpointRole::Client)
        .expect("encode request frame");
    assert_eq!(
        encoded,
        [
            0, 0, 0, 26, // frame_len: 24-byte header + 2-byte payload
            0, 1, // version
            3, // Request
            0, // flags
            0, 0, 0, 0, // control stream
            1, 2, 3, 4, 5, 6, 7, 8, // message_id
            17, 18, 19, 20, 21, 22, 23, 24, // sequence
            b'{', b'}',
        ]
    );

    let (decoded, consumed) = Frame::decode(&encoded, EndpointRole::Client)
        .expect("decode request frame")
        .expect("complete request frame");
    assert_eq!(decoded, frame);
    assert_eq!(consumed, encoded.len());
}

#[test]
fn decoder_accepts_preface_header_and_payload_one_byte_at_a_time() {
    let preface = Preface {
        major: WIRE_MAJOR,
        minor: WIRE_MINOR,
    };
    let frame = request_frame(1, br#"{"method":"session.list"}"#);
    let mut wire = preface.encode().to_vec();
    wire.extend_from_slice(
        &frame
            .encode(EndpointRole::Client)
            .expect("encode request frame"),
    );
    let mut decoder = FrameDecoder::new(EndpointRole::Client);
    let mut decoded = Vec::new();

    for byte in wire {
        decoded.extend(decoder.push(&[byte]).expect("decode byte"));
    }

    assert_eq!(decoder.preface(), Some(preface));
    assert_eq!(decoded, vec![frame]);
}

#[test]
fn decoder_emits_coalesced_frames_in_wire_order() {
    let preface = Preface {
        major: WIRE_MAJOR,
        minor: WIRE_MINOR,
    };
    let first = request_frame(1, br#"{"method":"session.list"}"#);
    let second = request_frame(2, br#"{"method":"state.snapshot"}"#);
    let mut wire = preface.encode().to_vec();
    wire.extend_from_slice(
        &first
            .encode(EndpointRole::Client)
            .expect("encode first request"),
    );
    wire.extend_from_slice(
        &second
            .encode(EndpointRole::Client)
            .expect("encode second request"),
    );
    let mut decoder = FrameDecoder::new(EndpointRole::Client);

    let decoded = decoder.push(&wire).expect("decode coalesced frames");

    assert_eq!(decoded, vec![first, second]);
}

#[test]
fn decoder_emits_many_coalesced_frames_in_wire_order() {
    let preface = Preface {
        major: WIRE_MAJOR,
        minor: WIRE_MINOR,
    };
    let frames = (1..=256)
        .map(|message_id| request_frame(message_id, br#"{"method":"session.list"}"#))
        .collect::<Vec<_>>();
    let mut wire = preface.encode().to_vec();
    for frame in &frames {
        wire.extend_from_slice(&frame.encode(EndpointRole::Client).expect("encode request"));
    }
    let mut decoder = FrameDecoder::new(EndpointRole::Client);

    let decoded = decoder.push(&wire).expect("decode coalesced frames");

    assert_eq!(decoded, frames);
}

#[test]
fn decoder_rejects_frame_length_smaller_than_header() {
    let encoded = (FRAME_HEADER_LEN as u32 - 1).to_be_bytes();

    assert!(Frame::decode(&encoded, EndpointRole::Client).is_err());
}

#[test]
fn decoder_rejects_frame_length_over_total_limit_before_payload_arrives() {
    let encoded = u32::try_from(MAX_FRAME_LEN + 1)
        .expect("frame limit fits u32")
        .to_be_bytes();

    assert!(Frame::decode(&encoded, EndpointRole::Client).is_err());
}

#[test]
fn decoder_rejects_unsupported_preface_major() {
    let encoded = Preface {
        major: WIRE_MAJOR + 1,
        minor: WIRE_MINOR,
    }
    .encode();

    assert!(Preface::decode(&encoded).is_err());
}

#[test]
fn codec_rejects_unsupported_frame_version() {
    let mut frame = request_frame(1, br#"{"method":"session.list"}"#);
    frame.header.version = WIRE_MAJOR + 1;

    assert!(frame.encode(EndpointRole::Client).is_err());
}

#[test]
fn decoder_rejects_unknown_frame_kind() {
    let mut encoded = request_frame(1, br#"{"method":"session.list"}"#)
        .encode(EndpointRole::Client)
        .expect("encode request");
    encoded[6] = 255;

    assert!(Frame::decode(&encoded, EndpointRole::Client).is_err());
}

#[test]
fn codec_rejects_nonzero_flags() {
    let mut frame = request_frame(1, br#"{"method":"session.list"}"#);
    frame.header.flags = 1;

    assert!(frame.encode(EndpointRole::Client).is_err());
}

#[test]
fn codec_rejects_control_json_over_limit() {
    let mut payload = vec![b'a'; MAX_CONTROL_PAYLOAD + 1];
    payload[0] = b'"';
    let last = payload.len() - 1;
    payload[last] = b'"';
    let frame = request_frame(1, &payload);

    assert!(frame.encode(EndpointRole::Client).is_err());
}

#[test]
fn codec_rejects_malformed_control_json() {
    let frame = request_frame(1, br#"{"method":"session.list""#);

    assert!(frame.encode(EndpointRole::Client).is_err());
}

#[test]
fn codec_rejects_output_payload_over_limit() {
    let frame = Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Output,
            flags: 0,
            stream_id: 1,
            message_id: 0,
            sequence: 1,
        },
        payload: vec![0; MAX_OUTPUT_PAYLOAD + 1],
    };

    assert!(frame.encode(EndpointRole::Server).is_err());
}

#[test]
fn codec_rejects_output_without_absolute_offset() {
    let frame = Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Output,
            flags: 0,
            stream_id: 1,
            message_id: 0,
            sequence: 1,
        },
        payload: vec![0; 7],
    };

    assert!(frame.encode(EndpointRole::Server).is_err());
}

#[test]
fn codec_rejects_zero_request_message_id() {
    let frame = request_frame(0, br#"{"method":"session.list"}"#);

    assert!(frame.encode(EndpointRole::Client).is_err());
}

#[test]
fn codec_enforces_stream_open_ownership() {
    let client_even = Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::StreamOpen,
            flags: 0,
            stream_id: 2,
            message_id: 0,
            sequence: 0,
        },
        payload: br#"{"kind":"events.v1","afterSeq":0}"#.to_vec(),
    };
    let server_odd = Frame {
        header: FrameHeader {
            stream_id: 1,
            ..client_even.header
        },
        payload: client_even.payload.clone(),
    };
    let client_control = Frame {
        header: FrameHeader {
            stream_id: 0,
            ..client_even.header
        },
        payload: client_even.payload.clone(),
    };

    assert!(client_even.encode(EndpointRole::Client).is_err());
    assert!(server_odd.encode(EndpointRole::Server).is_err());
    assert!(client_control.encode(EndpointRole::Client).is_err());
}

#[test]
fn codec_rejects_resize_payload_without_two_dimensions() {
    let frame = Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Resize,
            flags: 0,
            stream_id: 1,
            message_id: 0,
            sequence: 1,
        },
        payload: vec![0; 3],
    };

    assert!(frame.encode(EndpointRole::Client).is_err());
}

#[test]
fn codec_rejects_replay_marker_without_offset() {
    let frame = Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::ReplayBegin,
            flags: 0,
            stream_id: 1,
            message_id: 0,
            sequence: 1,
        },
        payload: vec![0; 7],
    };

    assert!(frame.encode(EndpointRole::Server).is_err());
}

#[test]
fn codec_rejects_ping_with_payload() {
    let frame = Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Ping,
            flags: 0,
            stream_id: 0,
            message_id: 0,
            sequence: 0,
        },
        payload: vec![0],
    };

    assert!(frame.encode(EndpointRole::Client).is_err());
}
