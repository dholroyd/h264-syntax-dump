use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process;

use h264_reader::Context;
use h264_reader::annexb::AnnexBReader;
use h264_reader::nal::pps::PicParameterSet;
use h264_reader::nal::sei::SeiReader;
use h264_reader::nal::slice::SliceHeader;
use h264_reader::nal::sps::SeqParameterSet;
use h264_reader::nal::sps_extension::SeqParameterSetExtension;
use h264_reader::nal::subset_sps::SubsetSps;
use h264_reader::nal::{Nal, NalHeader, RefNal, UnitType, parse_nal_header_extension};
use h264_reader::push::{AccumulatedNalHandler, NalInterest};

use h264_syntax_dump::{
    AudDescribe, PpsDescribe, SeiPayloadDescribe, SliceHeaderDescribe, SpsDescribe,
    SpsExtensionDescribe, SubsetSpsDescribe,
};
use mpeg_syntax_dump::{AnsiRenderer, SyntaxDescribe, SyntaxWrite};

struct NalCollector(Vec<Vec<u8>>);

impl AccumulatedNalHandler for &mut NalCollector {
    fn nal(&mut self, nal: RefNal<'_>) -> NalInterest {
        if nal.is_complete() {
            let mut bytes = Vec::new();
            std::io::Read::read_to_end(&mut nal.reader(), &mut bytes).unwrap();
            self.0.push(bytes);
        }
        NalInterest::Buffer
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file.264>", args[0]);
        process::exit(1);
    }
    let filename = &args[1];

    if let Err(e) = run(filename) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn run(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(filename)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let stdout = io::stdout();
    let mut renderer = AnsiRenderer::new(io::BufWriter::new(stdout.lock()));

    let mut ctx = Context::new();
    let mut sei_scratch = Vec::new();

    // Collect complete NALs by pushing data through the Annex B parser.
    let mut collector = NalCollector(Vec::new());
    {
        let mut reader = AnnexBReader::accumulate(&mut collector);
        reader.push(&data);
        reader.reset();
    }
    let nals = collector.0;

    for (idx, nal_bytes) in nals.iter().enumerate() {
        let context = format!("NAL #{} ({} bytes)", idx + 1, nal_bytes.len());
        describe_nal(
            nal_bytes,
            &context,
            &mut ctx,
            &mut renderer,
            &mut sei_scratch,
        )?;
    }

    Ok(())
}

fn hex_dump(data: &[u8], max_bytes: usize) -> String {
    let len = data.len().min(max_bytes);
    let mut lines = Vec::new();
    for chunk_start in (0..len).step_by(16) {
        let chunk_end = (chunk_start + 16).min(len);
        let hex: Vec<String> = data[chunk_start..chunk_end]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let ascii: String = data[chunk_start..chunk_end]
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();
        lines.push(format!(
            "  {chunk_start:04x}: {:<48} {ascii}",
            hex.join(" ")
        ));
    }
    if data.len() > max_bytes {
        lines.push(format!("  ... ({} bytes total)", data.len()));
    }
    lines.join("\n")
}

fn print_parse_error(what: &str, error: &dyn std::fmt::Debug, context: &str, nal_bytes: &[u8]) {
    eprintln!("Warning: failed to parse {what}: {error:?}");
    eprintln!("  Context: {context}");
    eprintln!("{}", hex_dump(nal_bytes, 64));
}

/// Describe a single NAL unit, dispatching by type.
fn describe_nal<W: SyntaxWrite>(
    nal_bytes: &[u8],
    context: &str,
    ctx: &mut Context,
    renderer: &mut W,
    sei_scratch: &mut Vec<u8>,
) -> Result<(), W::Error> {
    let header = match NalHeader::new(nal_bytes[0]) {
        Ok(h) => h,
        Err(_) => return Ok(()),
    };

    let nal = RefNal::new(nal_bytes, &[], true);

    match header.nal_unit_type() {
        UnitType::SeqParameterSet => match SeqParameterSet::from_bits(nal.rbsp_bits()) {
            Ok(sps) => {
                SpsDescribe(&sps).describe(renderer)?;
                ctx.put_seq_param_set(sps);
            }
            Err(e) => print_parse_error("SPS", &e, context, nal_bytes),
        },
        UnitType::PicParameterSet => match PicParameterSet::from_bits(ctx, nal.rbsp_bits()) {
            Ok(pps) => {
                let sps_id = pps.seq_parameter_set_id;
                if let Some(sps) = ctx.sps_by_id(sps_id) {
                    PpsDescribe { pps: &pps, sps }.describe(renderer)?;
                }
                ctx.put_pic_param_set(pps);
            }
            Err(e) => print_parse_error("PPS", &e, context, nal_bytes),
        },
        UnitType::AccessUnitDelimiter => {
            match h264_reader::nal::aud::AccessUnitDelimiter::from_bits(nal.rbsp_bits()) {
                Ok(aud) => {
                    AudDescribe(&aud).describe(renderer)?;
                }
                Err(e) => print_parse_error("AUD", &e, context, nal_bytes),
            }
        }
        UnitType::SEI => {
            sei_scratch.clear();
            let mut sei_reader = SeiReader::from_rbsp_bytes(nal.rbsp_bytes(), sei_scratch);
            while let Ok(Some(msg)) = sei_reader.next() {
                let desc = SeiPayloadDescribe {
                    payload_type: msg.payload_type,
                    payload: msg.payload,
                };
                desc.describe(renderer)?;
            }
        }
        UnitType::SeqParameterSetExtension => {
            match SeqParameterSetExtension::from_bits(nal.rbsp_bits()) {
                Ok(ext) => {
                    SpsExtensionDescribe(&ext).describe(renderer)?;
                }
                Err(e) => print_parse_error("SPS extension", &e, context, nal_bytes),
            }
        }
        UnitType::SubsetSeqParameterSet => match SubsetSps::from_bits(nal.rbsp_bits()) {
            Ok(subset_sps) => {
                SubsetSpsDescribe(&subset_sps).describe(renderer)?;
                ctx.put_subset_seq_param_set(subset_sps);
            }
            Err(e) => print_parse_error("subset SPS", &e, context, nal_bytes),
        },
        UnitType::SliceLayerWithoutPartitioningIdr
        | UnitType::SliceLayerWithoutPartitioningNonIdr => {
            match SliceHeader::from_bits(ctx, &mut nal.rbsp_bits(), header, None) {
                Ok((slice_header, sps, pps)) => {
                    let desc = SliceHeaderDescribe {
                        header: &slice_header,
                        sps,
                        pps,
                    };
                    desc.describe(renderer)?;
                }
                Err(e) => print_parse_error("slice header", &e, context, nal_bytes),
            }
        }
        UnitType::SliceExtension | UnitType::SliceExtensionViewComponent => {
            match parse_nal_header_extension(&nal) {
                Ok((ext, _rbsp)) => {
                    let mut bits = h264_reader::rbsp::BitReader::new(
                        h264_reader::nal::extended_rbsp_bytes(&nal),
                    );
                    match SliceHeader::from_bits(ctx, &mut bits, header, Some(&ext)) {
                        Ok((slice_header, sps, pps)) => {
                            let desc = SliceHeaderDescribe {
                                header: &slice_header,
                                sps,
                                pps,
                            };
                            desc.describe(renderer)?;
                        }
                        Err(e) => {
                            print_parse_error("slice extension header", &e, context, nal_bytes)
                        }
                    }
                }
                Err(e) => print_parse_error("NAL header extension", &e, context, nal_bytes),
            }
        }
        _ => {
            // Other NAL types: just note their presence
        }
    }

    Ok(())
}
