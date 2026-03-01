use h264_reader::nal::subset_sps::{
    MvcSpsExtension, MvcVuiParametersExtension, SubsetSpsExtension, SvcSpsExtension,
};
use mpeg_syntax_dump::{
    FixedWidthField, SyntaxDescribe, SyntaxWrite, TermAnnotation, Value, VariableLengthField,
};

use crate::sps::describe_hrd;
use crate::{SpsDescribe, SubsetSpsDescribe};

impl SyntaxDescribe for SubsetSpsDescribe<'_> {
    fn describe<W: SyntaxWrite>(&self, w: &mut W) -> Result<(), W::Error> {
        let subset = self.0;
        w.begin_element("subset_seq_parameter_set_rbsp", None)?;

        // Emit the base SPS via SpsDescribe
        SpsDescribe(&subset.sps).describe(w)?;

        let profile_idc: u8 = subset.sps.profile_idc.into();

        match &subset.extension {
            Some(SubsetSpsExtension::Svc(svc)) => {
                // bit_equal_to_one                          f(1)
                w.fixed_width_field(&FixedWidthField {
                    name: "bit_equal_to_one",
                    bits: 1,
                    descriptor: "f(1)",
                    value: Some(Value::Unsigned(1)),
                    comment: None,
                })?;
                describe_svc_extension(w, svc, &subset.sps)?;
            }
            Some(SubsetSpsExtension::Mvc {
                ext,
                mvc_vui_parameters,
            }) => {
                // bit_equal_to_one                          f(1)
                w.fixed_width_field(&FixedWidthField {
                    name: "bit_equal_to_one",
                    bits: 1,
                    descriptor: "f(1)",
                    value: Some(Value::Unsigned(1)),
                    comment: None,
                })?;
                describe_mvc_extension(w, ext)?;

                // mvc_vui_parameters_present_flag           u(1)
                let has_mvc_vui = mvc_vui_parameters.is_some();
                w.fixed_width_field(&FixedWidthField {
                    name: "mvc_vui_parameters_present_flag",
                    bits: 1,
                    descriptor: "u(1)",
                    value: Some(Value::Bool(has_mvc_vui)),
                    comment: None,
                })?;
                w.begin_if("mvc_vui_parameters_present_flag", &[], has_mvc_vui)?;
                if let Some(vui) = mvc_vui_parameters {
                    describe_mvc_vui_parameters_extension(w, vui)?;
                }
                w.end_if()?;
            }
            Some(SubsetSpsExtension::Mvcd) => {
                // bit_equal_to_one                          f(1)
                w.fixed_width_field(&FixedWidthField {
                    name: "bit_equal_to_one",
                    bits: 1,
                    descriptor: "f(1)",
                    value: Some(Value::Unsigned(1)),
                    comment: None,
                })?;
                w.comment(&format!(
                    "seq_parameter_set_mvcd_extension (profile_idc={profile_idc}, not parsed)"
                ))?;
            }
            None => {}
        }

        // additional_extension2_flag                        u(1)
        w.fixed_width_field(&FixedWidthField {
            name: "additional_extension2_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(subset.additional_extension2_flag)),
            comment: None,
        })?;

        w.end_element()
    }
}

fn describe_svc_extension<W: SyntaxWrite>(
    w: &mut W,
    svc: &SvcSpsExtension,
    sps: &h264_reader::nal::sps::SeqParameterSet,
) -> Result<(), W::Error> {
    w.begin_element("seq_parameter_set_svc_extension", None)?;

    let chroma_array_type = sps.chroma_info.chroma_array_type();

    // inter_layer_deblocking_filter_control_present_flag u(1)
    w.fixed_width_field(&FixedWidthField {
        name: "inter_layer_deblocking_filter_control_present_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(
            svc.inter_layer_deblocking_filter_control_present_flag,
        )),
        comment: None,
    })?;

    // extended_spatial_scalability_idc                   u(2)
    w.fixed_width_field(&FixedWidthField {
        name: "extended_spatial_scalability_idc",
        bits: 2,
        descriptor: "u(2)",
        value: Some(Value::Unsigned(svc.extended_spatial_scalability_idc as u64)),
        comment: None,
    })?;

    // if (ChromaArrayType == 1 || ChromaArrayType == 2)
    let chroma_12 = chroma_array_type == 1 || chroma_array_type == 2;
    w.begin_if(
        "ChromaArrayType == 1 || ChromaArrayType == 2",
        &[TermAnnotation {
            name: "ChromaArrayType",
            value: Value::Unsigned(chroma_array_type as u64),
        }],
        chroma_12,
    )?;
    if chroma_12 {
        w.fixed_width_field(&FixedWidthField {
            name: "chroma_phase_x_plus1_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(svc.chroma_phase_x_plus1_flag)),
            comment: None,
        })?;
    }
    w.end_if()?;

    // if (ChromaArrayType == 1)
    let chroma_1 = chroma_array_type == 1;
    w.begin_if(
        "ChromaArrayType == 1",
        &[TermAnnotation {
            name: "ChromaArrayType",
            value: Value::Unsigned(chroma_array_type as u64),
        }],
        chroma_1,
    )?;
    if chroma_1 {
        w.fixed_width_field(&FixedWidthField {
            name: "chroma_phase_y_plus1",
            bits: 2,
            descriptor: "u(2)",
            value: Some(Value::Unsigned(svc.chroma_phase_y_plus1 as u64)),
            comment: None,
        })?;
    }
    w.end_if()?;

    // if (extended_spatial_scalability_idc == 1)
    let ess_1 = svc.extended_spatial_scalability_idc == 1;
    w.begin_if(
        "extended_spatial_scalability_idc == 1",
        &[TermAnnotation {
            name: "extended_spatial_scalability_idc",
            value: Value::Unsigned(svc.extended_spatial_scalability_idc as u64),
        }],
        ess_1,
    )?;
    if ess_1 {
        // if (ChromaArrayType == 1 || ChromaArrayType == 2)
        w.begin_if(
            "ChromaArrayType == 1 || ChromaArrayType == 2",
            &[TermAnnotation {
                name: "ChromaArrayType",
                value: Value::Unsigned(chroma_array_type as u64),
            }],
            chroma_12,
        )?;
        if chroma_12 {
            w.fixed_width_field(&FixedWidthField {
                name: "seq_ref_layer_chroma_phase_x_plus1_flag",
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(svc.seq_ref_layer_chroma_phase_x_plus1_flag)),
                comment: None,
            })?;
        }
        w.end_if()?;

        // if (ChromaArrayType == 1)
        w.begin_if(
            "ChromaArrayType == 1",
            &[TermAnnotation {
                name: "ChromaArrayType",
                value: Value::Unsigned(chroma_array_type as u64),
            }],
            chroma_1,
        )?;
        if chroma_1 {
            w.fixed_width_field(&FixedWidthField {
                name: "seq_ref_layer_chroma_phase_y_plus1",
                bits: 2,
                descriptor: "u(2)",
                value: Some(Value::Unsigned(
                    svc.seq_ref_layer_chroma_phase_y_plus1 as u64,
                )),
                comment: None,
            })?;
        }
        w.end_if()?;

        // seq_scaled_ref_layer offsets                   se(v)
        w.variable_length_field(&VariableLengthField {
            name: "seq_scaled_ref_layer_left_offset",
            descriptor: "se(v)",
            value: Some(Value::Signed(svc.seq_scaled_ref_layer_left_offset as i64)),
            comment: None,
        })?;
        w.variable_length_field(&VariableLengthField {
            name: "seq_scaled_ref_layer_top_offset",
            descriptor: "se(v)",
            value: Some(Value::Signed(svc.seq_scaled_ref_layer_top_offset as i64)),
            comment: None,
        })?;
        w.variable_length_field(&VariableLengthField {
            name: "seq_scaled_ref_layer_right_offset",
            descriptor: "se(v)",
            value: Some(Value::Signed(svc.seq_scaled_ref_layer_right_offset as i64)),
            comment: None,
        })?;
        w.variable_length_field(&VariableLengthField {
            name: "seq_scaled_ref_layer_bottom_offset",
            descriptor: "se(v)",
            value: Some(Value::Signed(svc.seq_scaled_ref_layer_bottom_offset as i64)),
            comment: None,
        })?;
    }
    w.end_if()?;

    // seq_tcoeff_level_prediction_flag                  u(1)
    w.fixed_width_field(&FixedWidthField {
        name: "seq_tcoeff_level_prediction_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(svc.seq_tcoeff_level_prediction_flag)),
        comment: None,
    })?;

    // if (seq_tcoeff_level_prediction_flag)
    w.begin_if(
        "seq_tcoeff_level_prediction_flag",
        &[TermAnnotation {
            name: "seq_tcoeff_level_prediction_flag",
            value: Value::Bool(svc.seq_tcoeff_level_prediction_flag),
        }],
        svc.seq_tcoeff_level_prediction_flag,
    )?;
    if svc.seq_tcoeff_level_prediction_flag {
        w.fixed_width_field(&FixedWidthField {
            name: "adaptive_tcoeff_level_prediction_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(svc.adaptive_tcoeff_level_prediction_flag)),
            comment: None,
        })?;
    }
    w.end_if()?;

    // slice_header_restriction_flag                     u(1)
    w.fixed_width_field(&FixedWidthField {
        name: "slice_header_restriction_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(svc.slice_header_restriction_flag)),
        comment: None,
    })?;

    // svc_vui_parameters_present_flag                   u(1)
    w.fixed_width_field(&FixedWidthField {
        name: "svc_vui_parameters_present_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(svc.svc_vui_parameters_present_flag)),
        comment: None,
    })?;

    w.end_element()
}

fn describe_mvc_extension<W: SyntaxWrite>(
    w: &mut W,
    mvc: &MvcSpsExtension,
) -> Result<(), W::Error> {
    w.begin_element("seq_parameter_set_mvc_extension", None)?;

    let num_views_minus1 = mvc.views.len().saturating_sub(1) as u32;

    // num_views_minus1                                  ue(v)
    w.variable_length_field(&VariableLengthField {
        name: "num_views_minus1",
        descriptor: "ue(v)",
        value: Some(Value::Unsigned(num_views_minus1 as u64)),
        comment: None,
    })?;

    // view_id loop
    w.begin_for(
        "i = 0; i <= num_views_minus1; i++",
        &[TermAnnotation {
            name: "num_views_minus1",
            value: Value::Unsigned(num_views_minus1 as u64),
        }],
    )?;
    for (i, view) in mvc.views.iter().enumerate() {
        w.for_iteration("i", i as u64)?;
        w.variable_length_field(&VariableLengthField {
            name: &format!("view_id[{i}]"),
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(view.view_id as u64)),
            comment: None,
        })?;
    }
    w.end_for()?;

    // anchor refs loop (starts at i=1)
    w.begin_for(
        "i = 1; i <= num_views_minus1; i++",
        &[TermAnnotation {
            name: "num_views_minus1",
            value: Value::Unsigned(num_views_minus1 as u64),
        }],
    )?;
    for i in 1..mvc.views.len() {
        let view = &mvc.views[i];
        w.for_iteration("i", i as u64)?;

        // num_anchor_refs_l0                            ue(v)
        w.variable_length_field(&VariableLengthField {
            name: &format!("num_anchor_refs_l0[{i}]"),
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(view.anchor_refs_l0.len() as u64)),
            comment: None,
        })?;
        if !view.anchor_refs_l0.is_empty() {
            w.begin_for(
                &format!("j = 0; j < num_anchor_refs_l0[{i}]; j++"),
                &[TermAnnotation {
                    name: &format!("num_anchor_refs_l0[{i}]"),
                    value: Value::Unsigned(view.anchor_refs_l0.len() as u64),
                }],
            )?;
            for (j, ref_id) in view.anchor_refs_l0.iter().enumerate() {
                w.for_iteration("j", j as u64)?;
                w.variable_length_field(&VariableLengthField {
                    name: &format!("anchor_ref_l0[{i}][{j}]"),
                    descriptor: "ue(v)",
                    value: Some(Value::Unsigned(*ref_id as u64)),
                    comment: None,
                })?;
            }
            w.end_for()?;
        }

        // num_anchor_refs_l1                            ue(v)
        w.variable_length_field(&VariableLengthField {
            name: &format!("num_anchor_refs_l1[{i}]"),
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(view.anchor_refs_l1.len() as u64)),
            comment: None,
        })?;
        if !view.anchor_refs_l1.is_empty() {
            w.begin_for(
                &format!("j = 0; j < num_anchor_refs_l1[{i}]; j++"),
                &[TermAnnotation {
                    name: &format!("num_anchor_refs_l1[{i}]"),
                    value: Value::Unsigned(view.anchor_refs_l1.len() as u64),
                }],
            )?;
            for (j, ref_id) in view.anchor_refs_l1.iter().enumerate() {
                w.for_iteration("j", j as u64)?;
                w.variable_length_field(&VariableLengthField {
                    name: &format!("anchor_ref_l1[{i}][{j}]"),
                    descriptor: "ue(v)",
                    value: Some(Value::Unsigned(*ref_id as u64)),
                    comment: None,
                })?;
            }
            w.end_for()?;
        }
    }
    w.end_for()?;

    // non-anchor refs loop (starts at i=1)
    w.begin_for(
        "i = 1; i <= num_views_minus1; i++",
        &[TermAnnotation {
            name: "num_views_minus1",
            value: Value::Unsigned(num_views_minus1 as u64),
        }],
    )?;
    for i in 1..mvc.views.len() {
        let view = &mvc.views[i];
        w.for_iteration("i", i as u64)?;

        // num_non_anchor_refs_l0                        ue(v)
        w.variable_length_field(&VariableLengthField {
            name: &format!("num_non_anchor_refs_l0[{i}]"),
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(view.non_anchor_refs_l0.len() as u64)),
            comment: None,
        })?;
        if !view.non_anchor_refs_l0.is_empty() {
            w.begin_for(
                &format!("j = 0; j < num_non_anchor_refs_l0[{i}]; j++"),
                &[TermAnnotation {
                    name: &format!("num_non_anchor_refs_l0[{i}]"),
                    value: Value::Unsigned(view.non_anchor_refs_l0.len() as u64),
                }],
            )?;
            for (j, ref_id) in view.non_anchor_refs_l0.iter().enumerate() {
                w.for_iteration("j", j as u64)?;
                w.variable_length_field(&VariableLengthField {
                    name: &format!("non_anchor_ref_l0[{i}][{j}]"),
                    descriptor: "ue(v)",
                    value: Some(Value::Unsigned(*ref_id as u64)),
                    comment: None,
                })?;
            }
            w.end_for()?;
        }

        // num_non_anchor_refs_l1                        ue(v)
        w.variable_length_field(&VariableLengthField {
            name: &format!("num_non_anchor_refs_l1[{i}]"),
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(view.non_anchor_refs_l1.len() as u64)),
            comment: None,
        })?;
        if !view.non_anchor_refs_l1.is_empty() {
            w.begin_for(
                &format!("j = 0; j < num_non_anchor_refs_l1[{i}]; j++"),
                &[TermAnnotation {
                    name: &format!("num_non_anchor_refs_l1[{i}]"),
                    value: Value::Unsigned(view.non_anchor_refs_l1.len() as u64),
                }],
            )?;
            for (j, ref_id) in view.non_anchor_refs_l1.iter().enumerate() {
                w.for_iteration("j", j as u64)?;
                w.variable_length_field(&VariableLengthField {
                    name: &format!("non_anchor_ref_l1[{i}][{j}]"),
                    descriptor: "ue(v)",
                    value: Some(Value::Unsigned(*ref_id as u64)),
                    comment: None,
                })?;
            }
            w.end_for()?;
        }
    }
    w.end_for()?;

    // level values
    let num_level_values_minus1 = mvc.level_values.len().saturating_sub(1) as u32;
    w.variable_length_field(&VariableLengthField {
        name: "num_level_values_signalled_minus1",
        descriptor: "ue(v)",
        value: Some(Value::Unsigned(num_level_values_minus1 as u64)),
        comment: None,
    })?;

    w.begin_for(
        "i = 0; i <= num_level_values_signalled_minus1; i++",
        &[TermAnnotation {
            name: "num_level_values_signalled_minus1",
            value: Value::Unsigned(num_level_values_minus1 as u64),
        }],
    )?;
    for (i, lv) in mvc.level_values.iter().enumerate() {
        w.for_iteration("i", i as u64)?;

        // level_idc                                     u(8)
        w.fixed_width_field(&FixedWidthField {
            name: &format!("level_idc[{i}]"),
            bits: 8,
            descriptor: "u(8)",
            value: Some(Value::Unsigned(lv.level_idc as u64)),
            comment: None,
        })?;

        let num_ops_minus1 = lv.applicable_ops.len().saturating_sub(1) as u32;
        w.variable_length_field(&VariableLengthField {
            name: &format!("num_applicable_ops_minus1[{i}]"),
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(num_ops_minus1 as u64)),
            comment: None,
        })?;

        w.begin_for(
            &format!("j = 0; j <= num_applicable_ops_minus1[{i}]; j++"),
            &[TermAnnotation {
                name: &format!("num_applicable_ops_minus1[{i}]"),
                value: Value::Unsigned(num_ops_minus1 as u64),
            }],
        )?;
        for (j, op) in lv.applicable_ops.iter().enumerate() {
            w.for_iteration("j", j as u64)?;

            // applicable_op_temporal_id                 u(3)
            w.fixed_width_field(&FixedWidthField {
                name: &format!("applicable_op_temporal_id[{i}][{j}]"),
                bits: 3,
                descriptor: "u(3)",
                value: Some(Value::Unsigned(op.temporal_id as u64)),
                comment: None,
            })?;

            // applicable_op_num_target_views_minus1     ue(v)
            w.variable_length_field(&VariableLengthField {
                name: &format!("applicable_op_num_target_views_minus1[{i}][{j}]"),
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(op.num_target_views_minus1 as u64)),
                comment: None,
            })?;

            // target view ids loop
            w.begin_for(
                &format!("k = 0; k <= applicable_op_num_target_views_minus1[{i}][{j}]; k++"),
                &[TermAnnotation {
                    name: &format!("applicable_op_num_target_views_minus1[{i}][{j}]"),
                    value: Value::Unsigned(op.num_target_views_minus1 as u64),
                }],
            )?;
            for (k, view_id) in op.target_view_ids.iter().enumerate() {
                w.for_iteration("k", k as u64)?;
                w.variable_length_field(&VariableLengthField {
                    name: &format!("applicable_op_target_view_id[{i}][{j}][{k}]"),
                    descriptor: "ue(v)",
                    value: Some(Value::Unsigned(*view_id as u64)),
                    comment: None,
                })?;
            }
            w.end_for()?;

            // applicable_op_num_views_minus1            ue(v)
            w.variable_length_field(&VariableLengthField {
                name: &format!("applicable_op_num_views_minus1[{i}][{j}]"),
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(op.num_views_minus1 as u64)),
                comment: None,
            })?;
        }
        w.end_for()?;
    }
    w.end_for()?;

    w.end_element()
}

fn describe_mvc_vui_parameters_extension<W: SyntaxWrite>(
    w: &mut W,
    vui: &MvcVuiParametersExtension,
) -> Result<(), W::Error> {
    w.begin_element("mvc_vui_parameters_extension", None)?;

    let num_ops_minus1 = vui.ops.len().saturating_sub(1) as u32;

    // vui_mvc_num_ops_minus1                              ue(v)
    w.variable_length_field(&VariableLengthField {
        name: "vui_mvc_num_ops_minus1",
        descriptor: "ue(v)",
        value: Some(Value::Unsigned(num_ops_minus1 as u64)),
        comment: None,
    })?;

    w.begin_for(
        "i = 0; i <= vui_mvc_num_ops_minus1; i++",
        &[TermAnnotation {
            name: "vui_mvc_num_ops_minus1",
            value: Value::Unsigned(num_ops_minus1 as u64),
        }],
    )?;
    for (i, op) in vui.ops.iter().enumerate() {
        w.for_iteration("i", i as u64)?;

        // vui_mvc_temporal_id                             u(3)
        w.fixed_width_field(&FixedWidthField {
            name: &format!("vui_mvc_temporal_id[{i}]"),
            bits: 3,
            descriptor: "u(3)",
            value: Some(Value::Unsigned(op.temporal_id as u64)),
            comment: None,
        })?;

        let num_target_minus1 = op.target_output_view_ids.len().saturating_sub(1) as u32;

        // vui_mvc_num_target_output_views_minus1          ue(v)
        w.variable_length_field(&VariableLengthField {
            name: &format!("vui_mvc_num_target_output_views_minus1[{i}]"),
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(num_target_minus1 as u64)),
            comment: None,
        })?;

        w.begin_for(
            &format!("j = 0; j <= vui_mvc_num_target_output_views_minus1[{i}]; j++"),
            &[TermAnnotation {
                name: &format!("vui_mvc_num_target_output_views_minus1[{i}]"),
                value: Value::Unsigned(num_target_minus1 as u64),
            }],
        )?;
        for (j, view_id) in op.target_output_view_ids.iter().enumerate() {
            w.for_iteration("j", j as u64)?;
            w.variable_length_field(&VariableLengthField {
                name: &format!("vui_mvc_view_id[{i}][{j}]"),
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(*view_id as u64)),
                comment: None,
            })?;
        }
        w.end_for()?;

        // timing_info_present_flag                        u(1)
        let has_timing = op.timing_info.is_some();
        w.fixed_width_field(&FixedWidthField {
            name: "timing_info_present_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(has_timing)),
            comment: None,
        })?;
        w.begin_if("timing_info_present_flag", &[], has_timing)?;
        if let Some(ti) = &op.timing_info {
            w.fixed_width_field(&FixedWidthField {
                name: "num_units_in_tick",
                bits: 32,
                descriptor: "u(32)",
                value: Some(Value::Unsigned(ti.num_units_in_tick as u64)),
                comment: None,
            })?;
            w.fixed_width_field(&FixedWidthField {
                name: "time_scale",
                bits: 32,
                descriptor: "u(32)",
                value: Some(Value::Unsigned(ti.time_scale as u64)),
                comment: None,
            })?;
            w.fixed_width_field(&FixedWidthField {
                name: "fixed_frame_rate_flag",
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(ti.fixed_frame_rate_flag)),
                comment: None,
            })?;
        }
        w.end_if()?;

        // nal_hrd_parameters_present_flag                 u(1)
        let has_nal_hrd = op.nal_hrd_parameters.is_some();
        w.fixed_width_field(&FixedWidthField {
            name: "nal_hrd_parameters_present_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(has_nal_hrd)),
            comment: None,
        })?;
        w.begin_if("nal_hrd_parameters_present_flag", &[], has_nal_hrd)?;
        if let Some(hrd) = &op.nal_hrd_parameters {
            describe_hrd(w, hrd)?;
        }
        w.end_if()?;

        // vcl_hrd_parameters_present_flag                 u(1)
        let has_vcl_hrd = op.vcl_hrd_parameters.is_some();
        w.fixed_width_field(&FixedWidthField {
            name: "vcl_hrd_parameters_present_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(has_vcl_hrd)),
            comment: None,
        })?;
        w.begin_if("vcl_hrd_parameters_present_flag", &[], has_vcl_hrd)?;
        if let Some(hrd) = &op.vcl_hrd_parameters {
            describe_hrd(w, hrd)?;
        }
        w.end_if()?;

        // if (nal_hrd_parameters_present_flag || vcl_hrd_parameters_present_flag)
        let has_any_hrd = has_nal_hrd || has_vcl_hrd;
        w.begin_if(
            "nal_hrd_parameters_present_flag || vcl_hrd_parameters_present_flag",
            &[
                TermAnnotation {
                    name: "nal_hrd_parameters_present_flag",
                    value: Value::Bool(has_nal_hrd),
                },
                TermAnnotation {
                    name: "vcl_hrd_parameters_present_flag",
                    value: Value::Bool(has_vcl_hrd),
                },
            ],
            has_any_hrd,
        )?;
        if let Some(low_delay) = op.low_delay_hrd_flag {
            w.fixed_width_field(&FixedWidthField {
                name: "vui_mvc_low_delay_hrd_flag",
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(low_delay)),
                comment: None,
            })?;
        }
        w.end_if()?;

        // vui_mvc_pic_struct_present_flag                 u(1)
        w.fixed_width_field(&FixedWidthField {
            name: "vui_mvc_pic_struct_present_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(op.pic_struct_present_flag)),
            comment: None,
        })?;
    }
    w.end_for()?;

    w.end_element()
}
