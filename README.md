# h264-syntax-dump

Bridge crate that renders parsed [h264-reader](https://docs.rs/h264-reader) structures as human-readable syntax tables via [mpeg-syntax-dump](../mpeg-syntax-dump).

Rust's orphan rule prevents implementing a foreign trait on a foreign type directly. This crate provides newtype wrappers around h264-reader types and implements `SyntaxDescribe` on those wrappers.

## Supported NAL unit types

| Wrapper | Syntax table (ISO 14496-10) |
|---|---|
| `SpsDescribe` | `seq_parameter_set_data()` (7.3.2.1.1) |
| `PpsDescribe` | `pic_parameter_set_rbsp()` (7.3.2.2) |
| `SliceHeaderDescribe` | `slice_header()` (7.3.3) |
| `AudDescribe` | `access_unit_delimiter_rbsp()` (7.3.2.4) |
| `SeiPayloadDescribe` | SEI payload (hex dump) |
| `SpsExtensionDescribe` | `seq_parameter_set_extension_rbsp()` (7.3.2.1.2) |
| `SubsetSpsDescribe` | `subset_seq_parameter_set_rbsp()` (7.3.2.1.3) |

## Examples

### dump_from_annexb

Parse and dump all NAL units from a raw Annex B bitstream (start-code-delimited, no container):

```sh
cargo run --example dump_from_annexb -- input.264
```

### dump_sps

Parse and dump a hardcoded SPS (useful as a minimal usage example):

```sh
cargo run --example dump_sps
```

## Usage as a library

```rust
use h264_reader::nal::sps::SeqParameterSet;
use h264_reader::rbsp::BitReader;
use h264_syntax_dump::SpsDescribe;
use mpeg_syntax_dump::{AnsiRenderer, SyntaxDescribe};

let sps = SeqParameterSet::from_bits(BitReader::new(sps_bytes))?;
let mut renderer = AnsiRenderer::new(std::io::stdout().lock());
SpsDescribe(&sps).describe(&mut renderer)?;
```
