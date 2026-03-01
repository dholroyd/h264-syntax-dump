use h264_reader::nal::pps::{SliceGroup, SliceGroupChangeType};
use h264_reader::nal::sps::{ChromaFormat, ScalingList, SeqParameterSet};
use mpeg_syntax_dump::{
    FixedWidthField, SyntaxDescribe, SyntaxWrite, TermAnnotation, Value, VariableLengthField,
};

use crate::PpsDescribe;

impl SyntaxDescribe for PpsDescribe<'_> {
    fn describe<W: SyntaxWrite>(&self, w: &mut W) -> Result<(), W::Error> {
        let pps = self.pps;
        let sps = self.sps;
        w.begin_element("pic_parameter_set_rbsp", None)?;

        // pic_parameter_set_id                              ue(v)
        w.variable_length_field(&VariableLengthField {
            name: "pic_parameter_set_id",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(pps.pic_parameter_set_id.id() as u64)),
            comment: None,
        })?;

        // seq_parameter_set_id                              ue(v)
        w.variable_length_field(&VariableLengthField {
            name: "seq_parameter_set_id",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(pps.seq_parameter_set_id.id() as u64)),
            comment: None,
        })?;

        // entropy_coding_mode_flag                          u(1)
        w.fixed_width_field(&FixedWidthField {
            name: "entropy_coding_mode_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(pps.entropy_coding_mode_flag)),
            comment: None,
        })?;

        // bottom_field_pic_order_in_frame_present_flag      u(1)
        w.fixed_width_field(&FixedWidthField {
            name: "bottom_field_pic_order_in_frame_present_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(
                pps.bottom_field_pic_order_in_frame_present_flag,
            )),
            comment: None,
        })?;

        // num_slice_groups_minus1                           ue(v)
        let num_sg_minus1 = match &pps.slice_groups {
            Some(sg) => slice_group_num_minus1(sg),
            None => 0,
        };
        w.variable_length_field(&VariableLengthField {
            name: "num_slice_groups_minus1",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(num_sg_minus1 as u64)),
            comment: None,
        })?;

        // if (num_slice_groups_minus1 > 0) { ... }
        let has_sg = num_sg_minus1 > 0;
        w.begin_if(
            "num_slice_groups_minus1 > 0",
            &[TermAnnotation {
                name: "num_slice_groups_minus1",
                value: Value::Unsigned(num_sg_minus1 as u64),
            }],
            has_sg,
        )?;
        if let Some(sg) = &pps.slice_groups {
            describe_slice_groups(w, sg, num_sg_minus1)?;
        }
        w.end_if()?;

        // num_ref_idx_l0_default_active_minus1              ue(v)
        w.variable_length_field(&VariableLengthField {
            name: "num_ref_idx_l0_default_active_minus1",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(
                pps.num_ref_idx_l0_default_active_minus1 as u64,
            )),
            comment: None,
        })?;

        // num_ref_idx_l1_default_active_minus1              ue(v)
        w.variable_length_field(&VariableLengthField {
            name: "num_ref_idx_l1_default_active_minus1",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(
                pps.num_ref_idx_l1_default_active_minus1 as u64,
            )),
            comment: None,
        })?;

        // weighted_pred_flag                                u(1)
        w.fixed_width_field(&FixedWidthField {
            name: "weighted_pred_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(pps.weighted_pred_flag)),
            comment: None,
        })?;

        // weighted_bipred_idc                               u(2)
        w.fixed_width_field(&FixedWidthField {
            name: "weighted_bipred_idc",
            bits: 2,
            descriptor: "u(2)",
            value: Some(Value::Unsigned(pps.weighted_bipred_idc as u64)),
            comment: None,
        })?;

        // pic_init_qp_minus26                               se(v)
        w.variable_length_field(&VariableLengthField {
            name: "pic_init_qp_minus26",
            descriptor: "se(v)",
            value: Some(Value::Signed(pps.pic_init_qp_minus26 as i64)),
            comment: None,
        })?;

        // pic_init_qs_minus26                               se(v)
        w.variable_length_field(&VariableLengthField {
            name: "pic_init_qs_minus26",
            descriptor: "se(v)",
            value: Some(Value::Signed(pps.pic_init_qs_minus26 as i64)),
            comment: None,
        })?;

        // chroma_qp_index_offset                            se(v)
        w.variable_length_field(&VariableLengthField {
            name: "chroma_qp_index_offset",
            descriptor: "se(v)",
            value: Some(Value::Signed(pps.chroma_qp_index_offset as i64)),
            comment: None,
        })?;

        // deblocking_filter_control_present_flag            u(1)
        w.fixed_width_field(&FixedWidthField {
            name: "deblocking_filter_control_present_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(pps.deblocking_filter_control_present_flag)),
            comment: None,
        })?;

        // constrained_intra_pred_flag                       u(1)
        w.fixed_width_field(&FixedWidthField {
            name: "constrained_intra_pred_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(pps.constrained_intra_pred_flag)),
            comment: None,
        })?;

        // redundant_pic_cnt_present_flag                    u(1)
        w.fixed_width_field(&FixedWidthField {
            name: "redundant_pic_cnt_present_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(pps.redundant_pic_cnt_present_flag)),
            comment: None,
        })?;

        // if (more_rbsp_data()) — PPS extension
        let has_ext = pps.extension.is_some();
        w.begin_if("more_rbsp_data()", &[], has_ext)?;
        if let Some(ext) = &pps.extension {
            // transform_8x8_mode_flag                       u(1)
            w.fixed_width_field(&FixedWidthField {
                name: "transform_8x8_mode_flag",
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(ext.transform_8x8_mode_flag)),
                comment: None,
            })?;

            // pic_scaling_matrix_present_flag                u(1)
            let has_scaling = ext.pic_scaling_matrix.is_some();
            w.fixed_width_field(&FixedWidthField {
                name: "pic_scaling_matrix_present_flag",
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(has_scaling)),
                comment: None,
            })?;

            w.begin_if("pic_scaling_matrix_present_flag", &[], has_scaling)?;
            if let Some(psm) = &ext.pic_scaling_matrix {
                describe_pic_scaling_matrix(w, psm, sps, ext.transform_8x8_mode_flag)?;
            }
            w.end_if()?;

            // second_chroma_qp_index_offset                 se(v)
            w.variable_length_field(&VariableLengthField {
                name: "second_chroma_qp_index_offset",
                descriptor: "se(v)",
                value: Some(Value::Signed(ext.second_chroma_qp_index_offset as i64)),
                comment: None,
            })?;
        }
        w.end_if()?;

        w.end_element()
    }
}

fn slice_group_num_minus1(sg: &SliceGroup) -> u32 {
    match sg {
        SliceGroup::Interleaved { run_length_minus1 } => {
            run_length_minus1.len().saturating_sub(1) as u32
        }
        SliceGroup::Dispersed {
            num_slice_groups_minus1,
        } => *num_slice_groups_minus1,
        SliceGroup::ForegroundAndLeftover { rectangles } => rectangles.len() as u32,
        SliceGroup::Changing {
            num_slice_groups_minus1,
            ..
        } => *num_slice_groups_minus1,
        SliceGroup::ExplicitAssignment {
            num_slice_groups_minus1,
            ..
        } => *num_slice_groups_minus1,
    }
}

fn describe_slice_groups<W: SyntaxWrite>(
    w: &mut W,
    sg: &SliceGroup,
    num_sg_minus1: u32,
) -> Result<(), W::Error> {
    let map_type: u32 = match sg {
        SliceGroup::Interleaved { .. } => 0,
        SliceGroup::Dispersed { .. } => 1,
        SliceGroup::ForegroundAndLeftover { .. } => 2,
        SliceGroup::Changing { change_type, .. } => match change_type {
            SliceGroupChangeType::BoxOut => 3,
            SliceGroupChangeType::RasterScan => 4,
            SliceGroupChangeType::WipeOut => 5,
        },
        SliceGroup::ExplicitAssignment { .. } => 6,
    };

    // slice_group_map_type                              ue(v)
    w.variable_length_field(&VariableLengthField {
        name: "slice_group_map_type",
        descriptor: "ue(v)",
        value: Some(Value::Unsigned(map_type as u64)),
        comment: None,
    })?;

    // if (slice_group_map_type == 0)
    let is_0 = map_type == 0;
    w.begin_if(
        "slice_group_map_type == 0",
        &[TermAnnotation {
            name: "slice_group_map_type",
            value: Value::Unsigned(map_type as u64),
        }],
        is_0,
    )?;
    if let SliceGroup::Interleaved { run_length_minus1 } = sg {
        w.begin_for(
            "iGroup = 0; iGroup <= num_slice_groups_minus1; iGroup++",
            &[TermAnnotation {
                name: "num_slice_groups_minus1",
                value: Value::Unsigned(num_sg_minus1 as u64),
            }],
        )?;
        for (i, rl) in run_length_minus1.iter().enumerate() {
            w.for_iteration("iGroup", i as u64)?;
            w.variable_length_field(&VariableLengthField {
                name: &format!("run_length_minus1[{i}]"),
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(*rl as u64)),
                comment: None,
            })?;
        }
        w.end_for()?;
    }

    // else if (slice_group_map_type == 2)
    let is_2 = map_type == 2;
    w.begin_else_if("slice_group_map_type == 2", &[], is_2)?;
    if let SliceGroup::ForegroundAndLeftover { rectangles } = sg {
        w.begin_for(
            "iGroup = 0; iGroup < num_slice_groups_minus1; iGroup++",
            &[TermAnnotation {
                name: "num_slice_groups_minus1",
                value: Value::Unsigned(num_sg_minus1 as u64),
            }],
        )?;
        for i in 0..rectangles.len() {
            w.for_iteration("iGroup", i as u64)?;
            // SliceRect fields are private in h264-reader; emit a comment
            w.comment(&format!(
                "top_left[{i}] and bottom_right[{i}] (values not accessible)"
            ))?;
        }
        w.end_for()?;
    }

    // else if (slice_group_map_type == 3 || ... == 4 || ... == 5)
    let is_345 = matches!(map_type, 3..=5);
    w.begin_else_if(
        "slice_group_map_type == 3 || slice_group_map_type == 4 || slice_group_map_type == 5",
        &[],
        is_345,
    )?;
    if let SliceGroup::Changing {
        slice_group_change_direction_flag,
        slice_group_change_rate_minus1,
        ..
    } = sg
    {
        w.fixed_width_field(&FixedWidthField {
            name: "slice_group_change_direction_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(*slice_group_change_direction_flag)),
            comment: None,
        })?;
        w.variable_length_field(&VariableLengthField {
            name: "slice_group_change_rate_minus1",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(*slice_group_change_rate_minus1 as u64)),
            comment: None,
        })?;
    }

    // else if (slice_group_map_type == 6)
    let is_6 = map_type == 6;
    w.begin_else_if("slice_group_map_type == 6", &[], is_6)?;
    if let SliceGroup::ExplicitAssignment { slice_group_id, .. } = sg {
        let pic_size_minus1 = slice_group_id.len().saturating_sub(1) as u32;
        w.variable_length_field(&VariableLengthField {
            name: "pic_size_in_map_units_minus1",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(pic_size_minus1 as u64)),
            comment: None,
        })?;
        let bits = (1f64 + num_sg_minus1 as f64).log2().ceil() as u32;
        w.begin_for(
            "i = 0; i <= pic_size_in_map_units_minus1; i++",
            &[TermAnnotation {
                name: "pic_size_in_map_units_minus1",
                value: Value::Unsigned(pic_size_minus1 as u64),
            }],
        )?;
        for (i, id) in slice_group_id.iter().enumerate() {
            w.for_iteration("i", i as u64)?;
            w.fixed_width_field(&FixedWidthField {
                name: &format!("slice_group_id[{i}]"),
                bits,
                descriptor: &format!("u({bits})"),
                value: Some(Value::Unsigned(*id as u64)),
                comment: None,
            })?;
        }
        w.end_for()?;
    }
    w.end_if()?;

    Ok(())
}

fn describe_pic_scaling_matrix<W: SyntaxWrite>(
    w: &mut W,
    psm: &h264_reader::nal::pps::PicScalingMatrix,
    sps: &SeqParameterSet,
    transform_8x8_mode_flag: bool,
) -> Result<(), W::Error> {
    let extra_8x8 = if transform_8x8_mode_flag {
        if sps.chroma_info.chroma_format == ChromaFormat::YUV444 {
            6
        } else {
            2
        }
    } else {
        0
    };
    let total = 6 + extra_8x8;

    w.begin_for(
        "i = 0; i < 6 + ((chroma_format_idc != 3) ? 2 : 6) * transform_8x8_mode_flag; i++",
        &[TermAnnotation {
            name: "total",
            value: Value::Unsigned(total as u64),
        }],
    )?;

    for i in 0..total {
        w.for_iteration("i", i as u64)?;

        if i < 6 {
            let list = psm.scaling_list4x4.get(i as usize);
            let present = list.is_some_and(|l| !matches!(l, ScalingList::NotPresent));
            w.fixed_width_field(&FixedWidthField {
                name: &format!("pic_scaling_list_present_flag[{i}]"),
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(present)),
                comment: None,
            })?;
            w.begin_if(&format!("pic_scaling_list_present_flag[{i}]"), &[], present)?;
            if present
                && let Some(list) = list {
                    crate::sps::describe_scaling_list_4x4(w, list, i as u32)?;
                }
            w.end_if()?;
        } else {
            let idx = (i - 6) as usize;
            let list = psm.scaling_list8x8.as_ref().and_then(|v| v.get(idx));
            let present = list.is_some_and(|l| !matches!(l, ScalingList::NotPresent));
            w.fixed_width_field(&FixedWidthField {
                name: &format!("pic_scaling_list_present_flag[{i}]"),
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(present)),
                comment: None,
            })?;
            w.begin_if(&format!("pic_scaling_list_present_flag[{i}]"), &[], present)?;
            if present
                && let Some(list) = list {
                    crate::sps::describe_scaling_list_8x8(w, list, i as u32)?;
                }
            w.end_if()?;
        }
    }

    w.end_for()
}
