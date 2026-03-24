use h264_reader::nal::sei::HeaderType;
use mpeg_syntax_dump::{SyntaxDescribe, SyntaxWrite, VariableLengthField, Value};

use crate::SeiPayloadDescribe;

fn header_type_id(ht: HeaderType) -> u32 {
    match ht {
        HeaderType::BufferingPeriod => 0,
        HeaderType::PicTiming => 1,
        HeaderType::PanScanRect => 2,
        HeaderType::FillerPayload => 3,
        HeaderType::UserDataRegisteredItuTT35 => 4,
        HeaderType::UserDataUnregistered => 5,
        HeaderType::RecoveryPoint => 6,
        HeaderType::DecRefPicMarkingRepetition => 7,
        HeaderType::SparePic => 8,
        HeaderType::SceneInfo => 9,
        HeaderType::SubSeqInfo => 10,
        HeaderType::SubSeqLayerCharacteristics => 11,
        HeaderType::SubSeqCharacteristics => 12,
        HeaderType::FullFrameFreeze => 13,
        HeaderType::FullFrameFreezeRelease => 14,
        HeaderType::FullFrameSnapshot => 15,
        HeaderType::ProgressiveRefinementSegmentStart => 16,
        HeaderType::ProgressiveRefinementSegmentEnd => 17,
        HeaderType::MotionConstrainedSliceGroupSet => 18,
        HeaderType::FilmGrainCharacteristics => 19,
        HeaderType::DeblockingFilterDisplayPreference => 20,
        HeaderType::StereoVideoInfo => 21,
        HeaderType::PostFilterHint => 22,
        HeaderType::ToneMappingInfo => 23,
        HeaderType::ScalabilityInfo => 24,
        HeaderType::SubPicScalableLayer => 25,
        HeaderType::NonRequiredLayerRep => 26,
        HeaderType::PriorityLayerInfo => 27,
        HeaderType::LayersNotPresent => 28,
        HeaderType::LayerDependencyChange => 29,
        HeaderType::ScalableNesting => 30,
        HeaderType::BaseLayerTemporalHrd => 31,
        HeaderType::QualityLayerIntegrityCheck => 32,
        HeaderType::RedundantPicProperty => 33,
        HeaderType::Tl0DepRepIndex => 34,
        HeaderType::TlSwitchingPoint => 35,
        HeaderType::ParallelDecodingInfo => 36,
        HeaderType::MvcScalableNesting => 37,
        HeaderType::ViewScalabilityInfo => 38,
        HeaderType::MultiviewSceneInfo => 39,
        HeaderType::MultiviewAcquisitionInfo => 40,
        HeaderType::NonRequiredViewComponent => 41,
        HeaderType::ViewDependencyChange => 42,
        HeaderType::OperationPointsNotPresent => 43,
        HeaderType::BaseViewTemporalHrd => 44,
        HeaderType::FramePackingArrangement => 45,
        HeaderType::MultiviewViewPosition => 46,
        HeaderType::DisplayOrientation => 47,
        HeaderType::MvcdScalableNesting => 48,
        HeaderType::MvcdViewScalabilityInfo => 49,
        HeaderType::DepthRepresentationInfo => 50,
        HeaderType::ThreeDimensionalReferenceDisplaysInfo => 51,
        HeaderType::DepthTiming => 52,
        HeaderType::DepthSamplingInfo => 53,
        HeaderType::ConstrainedDepthParameterSetIdentifier => 54,
        HeaderType::GreenMetadata => 56,
        HeaderType::MasteringDisplayColourVolume => 137,
        HeaderType::ColourRemappingInfo => 142,
        HeaderType::AlternativeTransferCharacteristics => 147,
        HeaderType::AlternativeDepthInfo => 188,
        HeaderType::ReservedSeiMessage(id) => id,
    }
}

impl SyntaxDescribe for SeiPayloadDescribe<'_> {
    fn describe<W: SyntaxWrite>(&self, w: &mut W) -> Result<(), W::Error> {
        w.begin_element("sei_payload", None)?;

        // payloadType
        w.variable_length_field(&VariableLengthField {
            name: "payloadType",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(header_type_id(self.payload_type) as u64)),
            comment: Some(&format!("{:?}", self.payload_type)),
        })?;

        // payloadSize
        w.variable_length_field(&VariableLengthField {
            name: "payloadSize",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(self.payload.len() as u64)),
            comment: None,
        })?;

        match self.payload_type {
            HeaderType::RecoveryPoint => {
                // RecoveryPoint parsing not yet available in h264-reader
                if !self.payload.is_empty() {
                    w.raw_bytes(self.payload)?;
                }
            }
            _ => {
                if !self.payload.is_empty() {
                    w.raw_bytes(self.payload)?;
                }
            }
        }

        w.end_element()
    }
}
