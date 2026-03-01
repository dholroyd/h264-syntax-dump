use mpeg_syntax_dump::{
    FixedWidthField, SyntaxDescribe, SyntaxWrite, TermAnnotation, Value, VariableLengthField,
};

use crate::SpsExtensionDescribe;

impl SyntaxDescribe for SpsExtensionDescribe<'_> {
    fn describe<W: SyntaxWrite>(&self, w: &mut W) -> Result<(), W::Error> {
        let ext = self.0;
        w.begin_element("seq_parameter_set_extension_rbsp", None)?;

        // seq_parameter_set_id                              ue(v)
        w.variable_length_field(&VariableLengthField {
            name: "seq_parameter_set_id",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(ext.seq_parameter_set_id.id() as u64)),
            comment: None,
        })?;

        // aux_format_idc                                    ue(v)
        w.variable_length_field(&VariableLengthField {
            name: "aux_format_idc",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(ext.aux_format_idc as u64)),
            comment: None,
        })?;

        // if (aux_format_idc != 0)
        let has_aux = ext.aux_format_idc != 0;
        w.begin_if(
            "aux_format_idc != 0",
            &[TermAnnotation {
                name: "aux_format_idc",
                value: Value::Unsigned(ext.aux_format_idc as u64),
            }],
            has_aux,
        )?;
        if let Some(aux) = &ext.aux_format_info {
            // bit_depth_aux_minus8                          ue(v)
            w.variable_length_field(&VariableLengthField {
                name: "bit_depth_aux_minus8",
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(aux.bit_depth_aux_minus8 as u64)),
                comment: None,
            })?;

            // alpha_incr_flag                               u(1)
            w.fixed_width_field(&FixedWidthField {
                name: "alpha_incr_flag",
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(aux.alpha_incr_flag)),
                comment: None,
            })?;

            // alpha_opaque_value                            u(v)
            let alpha_bits = aux.bit_depth_aux_minus8 as u32 + 9;
            w.fixed_width_field(&FixedWidthField {
                name: "alpha_opaque_value",
                bits: alpha_bits,
                descriptor: &format!("u({alpha_bits})"),
                value: Some(Value::Unsigned(aux.alpha_opaque_value as u64)),
                comment: None,
            })?;

            // alpha_transparent_value                       u(v)
            w.fixed_width_field(&FixedWidthField {
                name: "alpha_transparent_value",
                bits: alpha_bits,
                descriptor: &format!("u({alpha_bits})"),
                value: Some(Value::Unsigned(aux.alpha_transparent_value as u64)),
                comment: None,
            })?;
        }
        w.end_if()?;

        // additional_extension_flag                         u(1)
        w.fixed_width_field(&FixedWidthField {
            name: "additional_extension_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(ext.additional_extension_flag)),
            comment: None,
        })?;

        w.end_element()
    }
}
