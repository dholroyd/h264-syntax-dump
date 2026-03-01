//! Bridge crate providing [`SyntaxDescribe`] implementations for
//! [h264-reader](https://docs.rs/h264-reader) parsed types.
//!
//! Rust's orphan rule prevents implementing a foreign trait on a foreign
//! type directly. This crate provides newtype wrappers around h264-reader
//! types and implements [`SyntaxDescribe`] on those wrappers, allowing
//! parsed H.264 structures to be rendered as human-readable syntax tables
//! via [`mpeg_syntax_dump`].

pub mod aud;
pub mod pps;
pub mod sei;
pub mod slice;
pub mod sps;
pub mod sps_extension;
pub mod subset_sps;

use h264_reader::nal::aud::AccessUnitDelimiter;
use h264_reader::nal::pps::PicParameterSet;
use h264_reader::nal::sei::HeaderType;
use h264_reader::nal::slice::SliceHeader;
use h264_reader::nal::sps::SeqParameterSet;
use h264_reader::nal::sps_extension::SeqParameterSetExtension;
use h264_reader::nal::subset_sps::SubsetSps;

/// Wrapper for describing an Access Unit Delimiter following the
/// `access_unit_delimiter_rbsp()` syntax table (ISO 14496-10, 7.3.2.4).
pub struct AudDescribe<'a>(pub &'a AccessUnitDelimiter);

/// Wrapper for describing a Sequence Parameter Set following the
/// `seq_parameter_set_data()` syntax table (ISO 14496-10, 7.3.2.1.1).
pub struct SpsDescribe<'a>(pub &'a SeqParameterSet);

/// Wrapper for describing a Picture Parameter Set following the
/// `pic_parameter_set_rbsp()` syntax table (ISO 14496-10, 7.3.2.2).
///
/// The SPS reference is needed for context-dependent conditionals
/// (e.g. `chroma_format_idc` affects scaling matrix size in the
/// PPS extension).
pub struct PpsDescribe<'a> {
    pub pps: &'a PicParameterSet,
    pub sps: &'a SeqParameterSet,
}

/// Wrapper for describing a Sequence Parameter Set Extension following the
/// `seq_parameter_set_extension_rbsp()` syntax table (ISO 14496-10, 7.3.2.1.2).
pub struct SpsExtensionDescribe<'a>(pub &'a SeqParameterSetExtension);

/// Wrapper for describing a Subset SPS following the
/// `subset_seq_parameter_set_rbsp()` syntax table (ISO 14496-10, 7.3.2.1.3).
pub struct SubsetSpsDescribe<'a>(pub &'a SubsetSps);

/// Wrapper for describing a raw SEI payload. Shows payload type, size,
/// and hex dump of the payload bytes.
pub struct SeiPayloadDescribe<'a> {
    pub payload_type: HeaderType,
    pub payload: &'a [u8],
}

/// Wrapper for describing a Slice Header following the `slice_header()`
/// syntax table (ISO 14496-10, 7.3.3).
///
/// Both SPS and PPS references are needed for context-dependent
/// conditionals throughout the slice header syntax.
pub struct SliceHeaderDescribe<'a> {
    pub header: &'a SliceHeader,
    pub sps: &'a SeqParameterSet,
    pub pps: &'a PicParameterSet,
}
