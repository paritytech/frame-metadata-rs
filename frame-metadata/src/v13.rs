// This file is part of Substrate.

// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::decode_different::*;
use codec::{Encode, Output};

cfg_if::cfg_if! {
	if #[cfg(feature = "std")] {
		use codec::Decode;
		use serde::Serialize;
	} else {
		extern crate alloc;
		use alloc::vec::Vec;
	}
}

/// Current prefix of metadata
pub const META_RESERVED: u32 = 0x6174656d; // 'meta' warn endianness

/// All the metadata about an outer event.
#[derive(Clone, PartialEq, Eq, Encode, Debug)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct OuterEventMetadata {
	pub name: DecodeDifferentStr,
	pub events: DecodeDifferentArray<
		(&'static str, FnEncode<&'static [EventMetadata]>),
		(StringBuf, Vec<EventMetadata>),
	>,
}

/// All the metadata about an event.
#[derive(Clone, PartialEq, Eq, Encode, Debug)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct EventMetadata {
	pub name: DecodeDifferentStr,
	pub arguments: DecodeDifferentArray<&'static str, StringBuf>,
	pub documentation: DecodeDifferentArray<&'static str, StringBuf>,
}

/// All the metadata about one storage entry.
#[derive(Clone, PartialEq, Eq, Encode, Debug)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct StorageEntryMetadata {
	pub name: DecodeDifferentStr,
	pub modifier: StorageEntryModifier,
	pub ty: StorageEntryType,
	pub default: ByteGetter,
	pub documentation: DecodeDifferentArray<&'static str, StringBuf>,
}

/// All the metadata about one module constant.
#[derive(Clone, PartialEq, Eq, Encode, Debug)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct ModuleConstantMetadata {
	pub name: DecodeDifferentStr,
	pub ty: DecodeDifferentStr,
	pub value: ByteGetter,
	pub documentation: DecodeDifferentArray<&'static str, StringBuf>,
}

/// All the metadata about a module error.
#[derive(Clone, PartialEq, Eq, Encode, Debug)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct ErrorMetadata {
	pub name: DecodeDifferentStr,
	pub documentation: DecodeDifferentArray<&'static str, StringBuf>,
}

/// All the metadata about errors in a module.
pub trait ModuleErrorMetadata {
	fn metadata() -> &'static [ErrorMetadata];
}

impl ModuleErrorMetadata for &'static str {
	fn metadata() -> &'static [ErrorMetadata] {
		&[]
	}
}

/// A technical trait to store lazy initiated vec value as static dyn pointer.
pub trait DefaultByte: Send + Sync {
	fn default_byte(&self) -> Vec<u8>;
}

/// Wrapper over dyn pointer for accessing a cached once byte value.
#[derive(Clone)]
pub struct DefaultByteGetter(pub &'static dyn DefaultByte);

/// Decode different for static lazy initiated byte value.
pub type ByteGetter = DecodeDifferent<DefaultByteGetter, Vec<u8>>;

impl Encode for DefaultByteGetter {
	fn encode_to<W: Output + ?Sized>(&self, dest: &mut W) {
		self.0.default_byte().encode_to(dest)
	}
}

impl codec::EncodeLike for DefaultByteGetter {}

impl PartialEq<DefaultByteGetter> for DefaultByteGetter {
	fn eq(&self, other: &DefaultByteGetter) -> bool {
		let left = self.0.default_byte();
		let right = other.0.default_byte();
		left.eq(&right)
	}
}

impl Eq for DefaultByteGetter {}

#[cfg(feature = "std")]
impl serde::Serialize for DefaultByteGetter {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.0.default_byte().serialize(serializer)
	}
}

impl core::fmt::Debug for DefaultByteGetter {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		self.0.default_byte().fmt(f)
	}
}

/// Hasher used by storage maps
#[derive(Clone, PartialEq, Eq, Encode, Debug)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub enum StorageHasher {
	Blake2_128,
	Blake2_256,
	Blake2_128Concat,
	Twox128,
	Twox256,
	Twox64Concat,
	Identity,
}

/// A storage entry type.
#[derive(Clone, PartialEq, Eq, Encode, Debug)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub enum StorageEntryType {
	Plain(DecodeDifferentStr),
	Map {
		hasher: StorageHasher,
		key: DecodeDifferentStr,
		value: DecodeDifferentStr,
		// is_linked flag previously, unused now to keep backwards compat
		unused: bool,
	},
	DoubleMap {
		hasher: StorageHasher,
		key1: DecodeDifferentStr,
		key2: DecodeDifferentStr,
		value: DecodeDifferentStr,
		key2_hasher: StorageHasher,
	},
	NMap {
		keys: DecodeDifferentArray<&'static str, StringBuf>,
		hashers: DecodeDifferentArray<StorageHasher>,
		value: DecodeDifferentStr,
	},
}

/// A storage entry modifier.
#[derive(Clone, PartialEq, Eq, Encode, Debug)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub enum StorageEntryModifier {
	Optional,
	Default,
}

/// All metadata of the storage.
#[derive(Clone, PartialEq, Eq, Encode, Debug)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct StorageMetadata {
	/// The common prefix used by all storage entries.
	pub prefix: DecodeDifferent<&'static str, StringBuf>,
	pub entries: DecodeDifferent<&'static [StorageEntryMetadata], Vec<StorageEntryMetadata>>,
}

/// Metadata of the extrinsic used by the runtime.
#[derive(Eq, Encode, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct ExtrinsicMetadata {
	/// Extrinsic version.
	pub version: u8,
	/// The signed extensions in the order they appear in the extrinsic.
	pub signed_extensions: Vec<DecodeDifferentStr>,
}

/// The metadata of a runtime.
#[derive(Eq, Encode, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct RuntimeMetadataV13 {
	/// Metadata of all the modules.
	pub modules: DecodeDifferentArray<ModuleMetadata>,
	/// Metadata of the extrinsic.
	pub extrinsic: ExtrinsicMetadata,
}

/// All metadata about an runtime module.
#[derive(Clone, PartialEq, Eq, Encode, Debug)]
#[cfg_attr(feature = "std", derive(Decode, Serialize))]
pub struct ModuleMetadata {
	pub name: DecodeDifferentStr,
	pub storage: Option<DecodeDifferent<FnEncode<StorageMetadata>, StorageMetadata>>,
	pub calls: ODFnA<FunctionMetadata>,
	pub event: ODFnA<EventMetadata>,
	pub constants: DFnA<ModuleConstantMetadata>,
	pub errors: DFnA<ErrorMetadata>,
	/// Define the index of the module, this index will be used for the encoding of module event,
	/// call and origin variants.
	pub index: u8,
}

type ODFnA<T> = Option<DFnA<T>>;
type DFnA<T> = DecodeDifferent<FnEncode<&'static [T]>, Vec<T>>;
