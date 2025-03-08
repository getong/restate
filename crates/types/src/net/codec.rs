// Copyright (c) 2023 - 2025 Restate Software, Inc., Restate GmbH.
// All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::sync::Arc;

use anyhow::Context;
use bytes::Buf;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::protobuf::common::ProtocolVersion;
use crate::protobuf::common::TargetName;

pub trait Targeted {
    const TARGET: TargetName;
    fn target(&self) -> TargetName {
        Self::TARGET
    }
    fn kind(&self) -> &'static str;
}

impl<T> Targeted for &T
where
    T: Targeted,
{
    const TARGET: TargetName = T::TARGET;
    fn kind(&self) -> &'static str {
        (*self).kind()
    }
}

impl<T> Targeted for &mut T
where
    T: Targeted,
{
    const TARGET: TargetName = T::TARGET;
    fn kind(&self) -> &'static str {
        (**self).kind()
    }
}

impl<T> Targeted for Box<T>
where
    T: Targeted,
{
    const TARGET: TargetName = T::TARGET;
    fn kind(&self) -> &'static str {
        (**self).kind()
    }
}

impl<T> Targeted for Arc<T>
where
    T: Targeted,
{
    const TARGET: TargetName = T::TARGET;
    fn kind(&self) -> &'static str {
        (**self).kind()
    }
}

pub trait WireEncode {
    fn encode_to_vec(self, protocol_version: ProtocolVersion) -> Vec<u8>;
}

pub trait WireDecode {
    fn decode(buf: impl Buf, protocol_version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl<T> WireEncode for Box<T>
where
    T: WireEncode,
{
    fn encode_to_vec(self, protocol_version: ProtocolVersion) -> Vec<u8> {
        (*self).encode_to_vec(protocol_version)
    }
}

impl<T> WireDecode for Box<T>
where
    T: WireDecode,
{
    fn decode(buf: impl Buf, protocol_version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Box::new(T::decode(buf, protocol_version)?))
    }
}

impl<T> WireDecode for Arc<T>
where
    T: WireDecode,
{
    fn decode(buf: impl Buf, protocol_version: ProtocolVersion) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Arc::new(T::decode(buf, protocol_version)?))
    }
}

/// Utility method to raw-encode a [`Serialize`] type as flexbuffers using serde without adding
/// version tag. This must be decoded with `decode_from_untagged_flexbuffers`. This is used as the default
/// encoding for network messages since networking has its own protocol versioning.
pub fn encode_default<T: Serialize>(value: T, protocol_version: ProtocolVersion) -> Vec<u8> {
    match protocol_version {
        ProtocolVersion::V1 => {
            flexbuffers::to_vec(value).expect("network message serde can't fail")
        }
        ProtocolVersion::Unknown => {
            unreachable!("unknown protocol version should never be set")
        }
    }
}

/// Utility method to decode a [`DeserializeOwned`] type from flexbuffers using serde. the buffer
/// must have the complete message and not internally chunked.
pub fn decode_default<T: DeserializeOwned>(
    buf: impl Buf,
    protocol_version: ProtocolVersion,
) -> anyhow::Result<T> {
    match protocol_version {
        ProtocolVersion::V1 => flexbuffers::from_slice(buf.chunk())
            .context("failed decoding V1 (flexbuffers) network message"),
        ProtocolVersion::Unknown => {
            unreachable!("unknown protocol version should never be set")
        }
    }
}
