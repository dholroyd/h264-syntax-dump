use h264_reader::nal::sei::recovery_point::RecoveryPoint;
use h264_reader::nal::sei::{HeaderType, SeiMessage};
use mpeg_syntax_dump::{FixedWidthField, SyntaxDescribe, SyntaxWrite, Value, VariableLengthField};

use crate::SeiPayloadDescribe;

impl SyntaxDescribe for SeiPayloadDescribe<'_> {
    fn describe<W: SyntaxWrite>(&self, w: &mut W) -> Result<(), W::Error> {
        w.begin_element("sei_payload", None)?;

        // payloadType
        w.variable_length_field(&VariableLengthField {
            name: "payloadType",
            descriptor: "ue(v)",
            value: None,
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
                let msg = SeiMessage {
                    payload_type: self.payload_type,
                    payload: self.payload,
                };
                if let Ok(rp) = RecoveryPoint::read(&msg) {
                    describe_recovery_point(w, &rp)?;
                } else if !self.payload.is_empty() {
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

fn describe_recovery_point<W: SyntaxWrite>(w: &mut W, rp: &RecoveryPoint) -> Result<(), W::Error> {
    w.variable_length_field(&VariableLengthField {
        name: "recovery_frame_cnt",
        descriptor: "ue(v)",
        value: Some(Value::Unsigned(rp.recovery_frame_cnt as u64)),
        comment: None,
    })?;
    w.fixed_width_field(&FixedWidthField {
        name: "exact_match_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Unsigned(rp.exact_match_flag as u64)),
        comment: None,
    })?;
    w.fixed_width_field(&FixedWidthField {
        name: "broken_link_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Unsigned(rp.broken_link_flag as u64)),
        comment: None,
    })?;
    w.fixed_width_field(&FixedWidthField {
        name: "changing_slice_group_idc",
        bits: 2,
        descriptor: "u(2)",
        value: Some(Value::Unsigned(rp.changing_slice_group_idc as u64)),
        comment: None,
    })?;
    Ok(())
}
