// Copyright (c) 2023 - 2025 Restate Software, Inc., Restate GmbH.
// All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

// [[NodeLocationScope]] specifies the location of a node in the cluster. The location
// is expressed by a set of hierarchical scopes. Restate assumes the cluster topology
// to be a tree-like structure: two nodes sharing a lower-level scope must
// be within the same higher level scope.
#[derive(
    Debug,
    Copy,
    Clone,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    strum::EnumIter,
    strum::Display,
    strum::EnumString,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
#[strum(ascii_case_insensitive)]
#[repr(u8)]
pub enum NodeLocationScope {
    /// Special; Indicating the smallest scope (an individual node)
    Node = 0,

    // Actua scopes representing the location of a node
    Zone,
    Region,

    // Special; Includes all lower-level scopes.
    Root,
}

impl NodeLocationScope {
    /// Returns None if self is already the largest scope, i.e. `Root`
    pub const fn next_greater_scope(self) -> Option<Self> {
        // we know `self + 1` won't overflow
        let next = (self as u8) + 1;
        Self::from_u8(next)
    }

    /// Returns None if self is already the smallest scope, i.e. `Node`
    pub const fn next_smaller_scope(self) -> Option<Self> {
        if matches!(self, Self::Node) {
            None
        } else {
            Self::from_u8(self as u8 - 1)
        }
    }

    pub const fn is_special(self) -> bool {
        matches!(self, NodeLocationScope::Root | NodeLocationScope::Node)
    }

    // Returns the number of non-special scopes.
    pub const fn num_scopes() -> usize {
        NodeLocationScope::Root as usize - 1
    }

    #[inline]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Node),
            1 => Some(Self::Zone),
            2 => Some(Self::Region),
            3 => Some(Self::Root),
            _ => None,
        }
    }
}
