use h264_reader::nal::sps::SeqParameterSet;
use h264_reader::rbsp::BitReader;
use h264_syntax_dump::SpsDescribe;
use mpeg_syntax_dump::{AnsiRenderer, SyntaxDescribe};

fn main() {
    // High profile SPS: 176x144 @ 10fps, with VUI timing info
    let sps_bytes: &[u8] = &[
        0x64, 0x00, 0x0A, 0xAC, 0x72, 0x84, 0x44, 0x26, 0x84, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00,
        0x00, 0xCA, 0x3C, 0x48, 0x96, 0x11, 0x80,
    ];
    let sps = SeqParameterSet::from_bits(BitReader::new(sps_bytes)).expect("SPS parse failed");
    let desc = SpsDescribe(&sps);

    let stdout = std::io::stdout();
    let mut renderer = AnsiRenderer::new(stdout.lock());
    desc.describe(&mut renderer).expect("render failed");
}
