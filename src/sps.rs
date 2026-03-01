use std::num::NonZeroU8;

use h264_reader::nal::sps::{
    AspectRatioInfo, ChromaInfo, FrameMbsFlags, HrdParameters, OverscanAppropriate,
    PicOrderCntType, ScalingList, SeqScalingMatrix, VuiParameters,
};
use mpeg_syntax_dump::{
    FixedWidthField, SyntaxDescribe, SyntaxWrite, TermAnnotation, Value, VariableLengthField,
};

use crate::SpsDescribe;

impl SyntaxDescribe for SpsDescribe<'_> {
    fn describe<W: SyntaxWrite>(&self, w: &mut W) -> Result<(), W::Error> {
        let sps = self.0;
        w.begin_element("seq_parameter_set_data", None)?;

        // profile_idc                                      u(8)
        let profile_idc_val = u8::from(sps.profile_idc);
        w.fixed_width_field(&FixedWidthField {
            name: "profile_idc",
            bits: 8,
            descriptor: "u(8)",
            value: Some(Value::Unsigned(profile_idc_val as u64)),
            comment: None,
        })?;

        // constraint_set0_flag .. constraint_set5_flag      u(1) each
        for i in 0..6 {
            let flag = match i {
                0 => sps.constraint_flags.flag0(),
                1 => sps.constraint_flags.flag1(),
                2 => sps.constraint_flags.flag2(),
                3 => sps.constraint_flags.flag3(),
                4 => sps.constraint_flags.flag4(),
                5 => sps.constraint_flags.flag5(),
                _ => unreachable!(),
            };
            w.fixed_width_field(&FixedWidthField {
                name: &format!("constraint_set{i}_flag"),
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(flag)),
                comment: None,
            })?;
        }

        // reserved_zero_2bits                               u(2)
        w.fixed_width_field(&FixedWidthField {
            name: "reserved_zero_2bits",
            bits: 2,
            descriptor: "u(2)",
            value: Some(Value::Unsigned(
                sps.constraint_flags.reserved_zero_two_bits() as u64,
            )),
            comment: None,
        })?;

        // level_idc                                         u(8)
        w.fixed_width_field(&FixedWidthField {
            name: "level_idc",
            bits: 8,
            descriptor: "u(8)",
            value: Some(Value::Unsigned(sps.level_idc as u64)),
            comment: None,
        })?;

        // seq_parameter_set_id                              ue(v)
        w.variable_length_field(&VariableLengthField {
            name: "seq_parameter_set_id",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(sps.seq_parameter_set_id.id() as u64)),
            comment: None,
        })?;

        // if (profile_idc == 100 || ...) — chroma info block
        let has_chroma = sps.profile_idc.has_chroma_info();
        w.begin_if(
            "profile_idc == 100 || profile_idc == 110 || profile_idc == 122 || profile_idc == 244 || profile_idc == 44 || profile_idc == 83 || profile_idc == 86 || profile_idc == 118 || profile_idc == 128 || profile_idc == 138 || profile_idc == 139 || profile_idc == 134 || profile_idc == 135",
            &[TermAnnotation {
                name: "profile_idc",
                value: Value::Unsigned(profile_idc_val as u64),
            }],
            has_chroma,
        )?;
        if has_chroma {
            describe_chroma_info(w, &sps.chroma_info)?;
        }
        w.end_if()?;

        // log2_max_frame_num_minus4                         ue(v)
        w.variable_length_field(&VariableLengthField {
            name: "log2_max_frame_num_minus4",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(sps.log2_max_frame_num_minus4 as u64)),
            comment: None,
        })?;

        // pic_order_cnt_type and its sub-fields
        describe_pic_order_cnt(w, &sps.pic_order_cnt)?;

        // max_num_ref_frames                                ue(v)
        w.variable_length_field(&VariableLengthField {
            name: "max_num_ref_frames",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(sps.max_num_ref_frames as u64)),
            comment: None,
        })?;

        // gaps_in_frame_num_value_allowed_flag              u(1)
        w.fixed_width_field(&FixedWidthField {
            name: "gaps_in_frame_num_value_allowed_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(sps.gaps_in_frame_num_value_allowed_flag)),
            comment: None,
        })?;

        // pic_width_in_mbs_minus1                           ue(v)
        w.variable_length_field(&VariableLengthField {
            name: "pic_width_in_mbs_minus1",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(sps.pic_width_in_mbs_minus1 as u64)),
            comment: None,
        })?;

        // pic_height_in_map_units_minus1                    ue(v)
        w.variable_length_field(&VariableLengthField {
            name: "pic_height_in_map_units_minus1",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(sps.pic_height_in_map_units_minus1 as u64)),
            comment: None,
        })?;

        // frame_mbs_only_flag                               u(1)
        let frame_mbs_only = matches!(sps.frame_mbs_flags, FrameMbsFlags::Frames);
        w.fixed_width_field(&FixedWidthField {
            name: "frame_mbs_only_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(frame_mbs_only)),
            comment: None,
        })?;

        // if (!frame_mbs_only_flag)
        w.begin_if(
            "!frame_mbs_only_flag",
            &[TermAnnotation {
                name: "frame_mbs_only_flag",
                value: Value::Bool(frame_mbs_only),
            }],
            !frame_mbs_only,
        )?;
        if let FrameMbsFlags::Fields {
            mb_adaptive_frame_field_flag,
        } = &sps.frame_mbs_flags
        {
            w.fixed_width_field(&FixedWidthField {
                name: "mb_adaptive_frame_field_flag",
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(*mb_adaptive_frame_field_flag)),
                comment: None,
            })?;
        }
        w.end_if()?;

        // direct_8x8_inference_flag                         u(1)
        w.fixed_width_field(&FixedWidthField {
            name: "direct_8x8_inference_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(sps.direct_8x8_inference_flag)),
            comment: None,
        })?;

        // frame_cropping_flag                               u(1)
        let cropping = sps.frame_cropping.is_some();
        w.fixed_width_field(&FixedWidthField {
            name: "frame_cropping_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(cropping)),
            comment: None,
        })?;

        // if (frame_cropping_flag)
        w.begin_if("frame_cropping_flag", &[], cropping)?;
        if let Some(crop) = &sps.frame_cropping {
            w.variable_length_field(&VariableLengthField {
                name: "frame_crop_left_offset",
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(crop.left_offset as u64)),
                comment: None,
            })?;
            w.variable_length_field(&VariableLengthField {
                name: "frame_crop_right_offset",
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(crop.right_offset as u64)),
                comment: None,
            })?;
            w.variable_length_field(&VariableLengthField {
                name: "frame_crop_top_offset",
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(crop.top_offset as u64)),
                comment: None,
            })?;
            w.variable_length_field(&VariableLengthField {
                name: "frame_crop_bottom_offset",
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(crop.bottom_offset as u64)),
                comment: None,
            })?;
        }
        w.end_if()?;

        // vui_parameters_present_flag                       u(1)
        let has_vui = sps.vui_parameters.is_some();
        w.fixed_width_field(&FixedWidthField {
            name: "vui_parameters_present_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(has_vui)),
            comment: None,
        })?;

        // if (vui_parameters_present_flag)
        w.begin_if("vui_parameters_present_flag", &[], has_vui)?;
        if let Some(vui) = &sps.vui_parameters {
            describe_vui(w, vui)?;
        }
        w.end_if()?;

        w.end_element()
    }
}

fn describe_chroma_info<W: SyntaxWrite>(w: &mut W, info: &ChromaInfo) -> Result<(), W::Error> {
    let chroma_format_idc = info.chroma_format.to_u32();

    // chroma_format_idc                                 ue(v)
    w.variable_length_field(&VariableLengthField {
        name: "chroma_format_idc",
        descriptor: "ue(v)",
        value: Some(Value::Unsigned(chroma_format_idc as u64)),
        comment: None,
    })?;

    // if (chroma_format_idc == 3)
    let is_444 = chroma_format_idc == 3;
    w.begin_if(
        "chroma_format_idc == 3",
        &[TermAnnotation {
            name: "chroma_format_idc",
            value: Value::Unsigned(chroma_format_idc as u64),
        }],
        is_444,
    )?;
    if is_444 {
        w.fixed_width_field(&FixedWidthField {
            name: "separate_colour_plane_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(info.separate_colour_plane_flag)),
            comment: None,
        })?;
    }
    w.end_if()?;

    // bit_depth_luma_minus8                             ue(v)
    w.variable_length_field(&VariableLengthField {
        name: "bit_depth_luma_minus8",
        descriptor: "ue(v)",
        value: Some(Value::Unsigned(info.bit_depth_luma_minus8 as u64)),
        comment: None,
    })?;

    // bit_depth_chroma_minus8                           ue(v)
    w.variable_length_field(&VariableLengthField {
        name: "bit_depth_chroma_minus8",
        descriptor: "ue(v)",
        value: Some(Value::Unsigned(info.bit_depth_chroma_minus8 as u64)),
        comment: None,
    })?;

    // qpprime_y_zero_transform_bypass_flag             u(1)
    w.fixed_width_field(&FixedWidthField {
        name: "qpprime_y_zero_transform_bypass_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(info.qpprime_y_zero_transform_bypass_flag)),
        comment: None,
    })?;

    // seq_scaling_matrix_present_flag                   u(1)
    let has_scaling = info.scaling_matrix.is_some();
    w.fixed_width_field(&FixedWidthField {
        name: "seq_scaling_matrix_present_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(has_scaling)),
        comment: None,
    })?;

    // if (seq_scaling_matrix_present_flag)
    w.begin_if("seq_scaling_matrix_present_flag", &[], has_scaling)?;
    if let Some(matrix) = &info.scaling_matrix {
        describe_seq_scaling_matrix(w, matrix, chroma_format_idc)?;
    }
    w.end_if()?;

    Ok(())
}

fn describe_seq_scaling_matrix<W: SyntaxWrite>(
    w: &mut W,
    matrix: &SeqScalingMatrix,
    chroma_format_idc: u32,
) -> Result<(), W::Error> {
    let count = if chroma_format_idc != 3 { 8 } else { 12 };
    let count_expr = if chroma_format_idc != 3 { "8" } else { "12" };
    w.begin_for(
        &format!("i = 0; i < {count_expr}; i++"),
        &[TermAnnotation {
            name: "chroma_format_idc",
            value: Value::Unsigned(chroma_format_idc as u64),
        }],
    )?;
    for i in 0..count {
        w.for_iteration("i", i as u64)?;

        if i < 6 {
            // 4x4 scaling list
            let list = matrix.scaling_list4x4.get(i as usize);
            let present = list.is_some_and(|l| !matches!(l, ScalingList::NotPresent));
            w.fixed_width_field(&FixedWidthField {
                name: &format!("seq_scaling_list_present_flag[{i}]"),
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(present)),
                comment: None,
            })?;
            w.begin_if(&format!("seq_scaling_list_present_flag[{i}]"), &[], present)?;
            if present
                && let Some(list) = list {
                    describe_scaling_list_4x4(w, list, i)?;
                }
            w.end_if()?;
        } else {
            // 8x8 scaling list
            let idx = (i - 6) as usize;
            let list = matrix.scaling_list8x8.get(idx);
            let present = list.is_some_and(|l| !matches!(l, ScalingList::NotPresent));
            w.fixed_width_field(&FixedWidthField {
                name: &format!("seq_scaling_list_present_flag[{i}]"),
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(present)),
                comment: None,
            })?;
            w.begin_if(&format!("seq_scaling_list_present_flag[{i}]"), &[], present)?;
            if present
                && let Some(list) = list {
                    describe_scaling_list_8x8(w, list, i)?;
                }
            w.end_if()?;
        }
    }
    w.end_for()
}

pub(crate) fn describe_scaling_list_4x4<W: SyntaxWrite>(
    w: &mut W,
    list: &ScalingList<16>,
    index: u32,
) -> Result<(), W::Error> {
    w.begin_element(
        "scaling_list",
        Some(&format!(
            "ScalingList4x4[{index}], 16, UseDefaultScalingMatrix4x4Flag[{index}]"
        )),
    )?;
    describe_scaling_list_body::<W, 16>(w, list)?;
    w.end_element()
}

pub(crate) fn describe_scaling_list_8x8<W: SyntaxWrite>(
    w: &mut W,
    list: &ScalingList<64>,
    index: u32,
) -> Result<(), W::Error> {
    let idx8 = index - 6;
    w.begin_element(
        "scaling_list",
        Some(&format!(
            "ScalingList8x8[{idx8}], 64, UseDefaultScalingMatrix8x8Flag[{idx8}]"
        )),
    )?;
    describe_scaling_list_body::<W, 64>(w, list)?;
    w.end_element()
}

fn describe_scaling_list_body<W: SyntaxWrite, const S: usize>(
    w: &mut W,
    list: &ScalingList<S>,
) -> Result<(), W::Error> {
    match list {
        ScalingList::NotPresent => {
            // Should not be called for NotPresent (filtered upstream)
        }
        ScalingList::UseDefault => {
            // UseDefault: emit a single delta_scale = -8 so nextScale becomes 0
            // at j=0, setting useDefaultScalingMatrixFlag = 1
            w.begin_for(&format!("j = 0; j < {S}; j++"), &[])?;
            w.for_iteration("j", 0)?;
            w.begin_if("nextScale != 0", &[], true)?;
            w.variable_length_field(&VariableLengthField {
                name: "delta_scale",
                descriptor: "se(v)",
                value: Some(Value::Signed(-8)),
                comment: Some("useDefaultScalingMatrixFlag = 1"),
            })?;
            w.end_if()?;
            w.end_for()?;
        }
        ScalingList::List(values) => {
            let deltas = compute_scaling_deltas(values);
            w.begin_for(&format!("j = 0; j < {S}; j++"), &[])?;
            let mut next_scale: i32 = 8;
            for (j, delta) in deltas.iter().enumerate() {
                w.for_iteration("j", j as u64)?;
                let reading = next_scale != 0;
                w.begin_if(
                    "nextScale != 0",
                    &[TermAnnotation {
                        name: "nextScale",
                        value: Value::Signed(next_scale as i64),
                    }],
                    reading,
                )?;
                if reading {
                    w.variable_length_field(&VariableLengthField {
                        name: "delta_scale",
                        descriptor: "se(v)",
                        value: Some(Value::Signed(*delta as i64)),
                        comment: None,
                    })?;
                    let last_scale = if j == 0 {
                        8
                    } else {
                        values[j - 1].get() as i32
                    };
                    next_scale = (last_scale + delta + 256) % 256;
                }
                w.end_if()?;
            }
            w.end_for()?;
        }
    }
    Ok(())
}

/// Compute delta_scale values that reproduce the given scaling list.
fn compute_scaling_deltas<const S: usize>(values: &[NonZeroU8; S]) -> Vec<i32> {
    let mut deltas = Vec::with_capacity(S);
    let mut last_scale: i32 = 8;
    for value in values {
        let target = value.get() as i32;
        let mut delta = target - last_scale;
        // Normalize to [-128, 127]
        while delta > 127 {
            delta -= 256;
        }
        while delta < -128 {
            delta += 256;
        }
        deltas.push(delta);
        let next_scale = (last_scale + delta + 256) % 256;
        // Since values are NonZeroU8, next_scale is always > 0
        last_scale = next_scale;
    }
    deltas
}

fn describe_pic_order_cnt<W: SyntaxWrite>(
    w: &mut W,
    poc: &PicOrderCntType,
) -> Result<(), W::Error> {
    let poc_type: u32 = match poc {
        PicOrderCntType::TypeZero { .. } => 0,
        PicOrderCntType::TypeOne { .. } => 1,
        PicOrderCntType::TypeTwo => 2,
    };

    // pic_order_cnt_type                                ue(v)
    w.variable_length_field(&VariableLengthField {
        name: "pic_order_cnt_type",
        descriptor: "ue(v)",
        value: Some(Value::Unsigned(poc_type as u64)),
        comment: None,
    })?;

    // if (pic_order_cnt_type == 0)
    let is_zero = poc_type == 0;
    w.begin_if(
        "pic_order_cnt_type == 0",
        &[TermAnnotation {
            name: "pic_order_cnt_type",
            value: Value::Unsigned(poc_type as u64),
        }],
        is_zero,
    )?;
    if let PicOrderCntType::TypeZero {
        log2_max_pic_order_cnt_lsb_minus4,
    } = poc
    {
        w.variable_length_field(&VariableLengthField {
            name: "log2_max_pic_order_cnt_lsb_minus4",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(*log2_max_pic_order_cnt_lsb_minus4 as u64)),
            comment: None,
        })?;
    }

    // else if (pic_order_cnt_type == 1)
    let is_one = poc_type == 1;
    w.begin_else_if("pic_order_cnt_type == 1", &[], is_one)?;
    if let PicOrderCntType::TypeOne {
        delta_pic_order_always_zero_flag,
        offset_for_non_ref_pic,
        offset_for_top_to_bottom_field,
        offsets_for_ref_frame,
    } = poc
    {
        w.fixed_width_field(&FixedWidthField {
            name: "delta_pic_order_always_zero_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(*delta_pic_order_always_zero_flag)),
            comment: None,
        })?;
        w.variable_length_field(&VariableLengthField {
            name: "offset_for_non_ref_pic",
            descriptor: "se(v)",
            value: Some(Value::Signed(*offset_for_non_ref_pic as i64)),
            comment: None,
        })?;
        w.variable_length_field(&VariableLengthField {
            name: "offset_for_top_to_bottom_field",
            descriptor: "se(v)",
            value: Some(Value::Signed(*offset_for_top_to_bottom_field as i64)),
            comment: None,
        })?;

        let num_ref = offsets_for_ref_frame.len() as u32;
        w.variable_length_field(&VariableLengthField {
            name: "num_ref_frames_in_pic_order_cnt_cycle",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(num_ref as u64)),
            comment: None,
        })?;

        w.begin_for(
            "i = 0; i < num_ref_frames_in_pic_order_cnt_cycle; i++",
            &[TermAnnotation {
                name: "num_ref_frames_in_pic_order_cnt_cycle",
                value: Value::Unsigned(num_ref as u64),
            }],
        )?;
        for (i, offset) in offsets_for_ref_frame.iter().enumerate() {
            w.for_iteration("i", i as u64)?;
            w.variable_length_field(&VariableLengthField {
                name: &format!("offset_for_ref_frame[{i}]"),
                descriptor: "se(v)",
                value: Some(Value::Signed(*offset as i64)),
                comment: None,
            })?;
        }
        w.end_for()?;
    }
    w.end_if()?;

    Ok(())
}

fn describe_vui<W: SyntaxWrite>(w: &mut W, vui: &VuiParameters) -> Result<(), W::Error> {
    w.begin_element("vui_parameters", None)?;

    // aspect_ratio_info_present_flag                    u(1)
    let has_aspect = vui.aspect_ratio_info.is_some();
    w.fixed_width_field(&FixedWidthField {
        name: "aspect_ratio_info_present_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(has_aspect)),
        comment: None,
    })?;
    w.begin_if("aspect_ratio_info_present_flag", &[], has_aspect)?;
    if let Some(ari) = &vui.aspect_ratio_info {
        let idc = ari.to_u8();
        w.fixed_width_field(&FixedWidthField {
            name: "aspect_ratio_idc",
            bits: 8,
            descriptor: "u(8)",
            value: Some(Value::Unsigned(idc as u64)),
            comment: None,
        })?;
        let is_extended = matches!(ari, AspectRatioInfo::Extended(_, _));
        w.begin_if(
            "aspect_ratio_idc == Extended_SAR",
            &[TermAnnotation {
                name: "aspect_ratio_idc",
                value: Value::Unsigned(idc as u64),
            }],
            is_extended,
        )?;
        if let AspectRatioInfo::Extended(sar_w, sar_h) = ari {
            w.fixed_width_field(&FixedWidthField {
                name: "sar_width",
                bits: 16,
                descriptor: "u(16)",
                value: Some(Value::Unsigned(*sar_w as u64)),
                comment: None,
            })?;
            w.fixed_width_field(&FixedWidthField {
                name: "sar_height",
                bits: 16,
                descriptor: "u(16)",
                value: Some(Value::Unsigned(*sar_h as u64)),
                comment: None,
            })?;
        }
        w.end_if()?;
    }
    w.end_if()?;

    // overscan_info_present_flag                        u(1)
    let has_overscan = !matches!(vui.overscan_appropriate, OverscanAppropriate::Unspecified);
    w.fixed_width_field(&FixedWidthField {
        name: "overscan_info_present_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(has_overscan)),
        comment: None,
    })?;
    w.begin_if("overscan_info_present_flag", &[], has_overscan)?;
    if has_overscan {
        let appropriate = matches!(vui.overscan_appropriate, OverscanAppropriate::Appropriate);
        w.fixed_width_field(&FixedWidthField {
            name: "overscan_appropriate_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(appropriate)),
            comment: None,
        })?;
    }
    w.end_if()?;

    // video_signal_type_present_flag                    u(1)
    let has_vst = vui.video_signal_type.is_some();
    w.fixed_width_field(&FixedWidthField {
        name: "video_signal_type_present_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(has_vst)),
        comment: None,
    })?;
    w.begin_if("video_signal_type_present_flag", &[], has_vst)?;
    if let Some(vst) = &vui.video_signal_type {
        w.fixed_width_field(&FixedWidthField {
            name: "video_format",
            bits: 3,
            descriptor: "u(3)",
            value: Some(Value::Unsigned(vst.video_format.to_u8() as u64)),
            comment: None,
        })?;
        w.fixed_width_field(&FixedWidthField {
            name: "video_full_range_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(vst.video_full_range_flag)),
            comment: None,
        })?;
        let has_colour = vst.colour_description.is_some();
        w.fixed_width_field(&FixedWidthField {
            name: "colour_description_present_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(has_colour)),
            comment: None,
        })?;
        w.begin_if("colour_description_present_flag", &[], has_colour)?;
        if let Some(cd) = &vst.colour_description {
            w.fixed_width_field(&FixedWidthField {
                name: "colour_primaries",
                bits: 8,
                descriptor: "u(8)",
                value: Some(Value::Unsigned(cd.colour_primaries as u64)),
                comment: None,
            })?;
            w.fixed_width_field(&FixedWidthField {
                name: "transfer_characteristics",
                bits: 8,
                descriptor: "u(8)",
                value: Some(Value::Unsigned(cd.transfer_characteristics as u64)),
                comment: None,
            })?;
            w.fixed_width_field(&FixedWidthField {
                name: "matrix_coefficients",
                bits: 8,
                descriptor: "u(8)",
                value: Some(Value::Unsigned(cd.matrix_coefficients as u64)),
                comment: None,
            })?;
        }
        w.end_if()?;
    }
    w.end_if()?;

    // chroma_loc_info_present_flag                      u(1)
    let has_chroma_loc = vui.chroma_loc_info.is_some();
    w.fixed_width_field(&FixedWidthField {
        name: "chroma_loc_info_present_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(has_chroma_loc)),
        comment: None,
    })?;
    w.begin_if("chroma_loc_info_present_flag", &[], has_chroma_loc)?;
    if let Some(cli) = &vui.chroma_loc_info {
        w.variable_length_field(&VariableLengthField {
            name: "chroma_sample_loc_type_top_field",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(cli.chroma_sample_loc_type_top_field as u64)),
            comment: None,
        })?;
        w.variable_length_field(&VariableLengthField {
            name: "chroma_sample_loc_type_bottom_field",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(
                cli.chroma_sample_loc_type_bottom_field as u64,
            )),
            comment: None,
        })?;
    }
    w.end_if()?;

    // timing_info_present_flag                          u(1)
    let has_timing = vui.timing_info.is_some();
    w.fixed_width_field(&FixedWidthField {
        name: "timing_info_present_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(has_timing)),
        comment: None,
    })?;
    w.begin_if("timing_info_present_flag", &[], has_timing)?;
    if let Some(ti) = &vui.timing_info {
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

    // nal_hrd_parameters_present_flag                   u(1)
    let has_nal_hrd = vui.nal_hrd_parameters.is_some();
    w.fixed_width_field(&FixedWidthField {
        name: "nal_hrd_parameters_present_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(has_nal_hrd)),
        comment: None,
    })?;
    w.begin_if("nal_hrd_parameters_present_flag", &[], has_nal_hrd)?;
    if let Some(hrd) = &vui.nal_hrd_parameters {
        describe_hrd(w, hrd)?;
    }
    w.end_if()?;

    // vcl_hrd_parameters_present_flag                   u(1)
    let has_vcl_hrd = vui.vcl_hrd_parameters.is_some();
    w.fixed_width_field(&FixedWidthField {
        name: "vcl_hrd_parameters_present_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(has_vcl_hrd)),
        comment: None,
    })?;
    w.begin_if("vcl_hrd_parameters_present_flag", &[], has_vcl_hrd)?;
    if let Some(hrd) = &vui.vcl_hrd_parameters {
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
    if let Some(low_delay) = vui.low_delay_hrd_flag {
        w.fixed_width_field(&FixedWidthField {
            name: "low_delay_hrd_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(low_delay)),
            comment: None,
        })?;
    }
    w.end_if()?;

    // pic_struct_present_flag                           u(1)
    w.fixed_width_field(&FixedWidthField {
        name: "pic_struct_present_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(vui.pic_struct_present_flag)),
        comment: None,
    })?;

    // bitstream_restriction_flag                        u(1)
    let has_bsr = vui.bitstream_restrictions.is_some();
    w.fixed_width_field(&FixedWidthField {
        name: "bitstream_restriction_flag",
        bits: 1,
        descriptor: "u(1)",
        value: Some(Value::Bool(has_bsr)),
        comment: None,
    })?;
    w.begin_if("bitstream_restriction_flag", &[], has_bsr)?;
    if let Some(br) = &vui.bitstream_restrictions {
        w.fixed_width_field(&FixedWidthField {
            name: "motion_vectors_over_pic_boundaries_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(br.motion_vectors_over_pic_boundaries_flag)),
            comment: None,
        })?;
        w.variable_length_field(&VariableLengthField {
            name: "max_bytes_per_pic_denom",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(br.max_bytes_per_pic_denom as u64)),
            comment: None,
        })?;
        w.variable_length_field(&VariableLengthField {
            name: "max_bits_per_mb_denom",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(br.max_bits_per_mb_denom as u64)),
            comment: None,
        })?;
        w.variable_length_field(&VariableLengthField {
            name: "log2_max_mv_length_horizontal",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(br.log2_max_mv_length_horizontal as u64)),
            comment: None,
        })?;
        w.variable_length_field(&VariableLengthField {
            name: "log2_max_mv_length_vertical",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(br.log2_max_mv_length_vertical as u64)),
            comment: None,
        })?;
        w.variable_length_field(&VariableLengthField {
            name: "max_num_reorder_frames",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(br.max_num_reorder_frames as u64)),
            comment: None,
        })?;
        w.variable_length_field(&VariableLengthField {
            name: "max_dec_frame_buffering",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(br.max_dec_frame_buffering as u64)),
            comment: None,
        })?;
    }
    w.end_if()?;

    w.end_element()
}

pub(crate) fn describe_hrd<W: SyntaxWrite>(w: &mut W, hrd: &HrdParameters) -> Result<(), W::Error> {
    w.begin_element("hrd_parameters", None)?;

    let cpb_cnt_minus1 = hrd.cpb_specs.len().saturating_sub(1) as u32;

    // cpb_cnt_minus1                                    ue(v)
    w.variable_length_field(&VariableLengthField {
        name: "cpb_cnt_minus1",
        descriptor: "ue(v)",
        value: Some(Value::Unsigned(cpb_cnt_minus1 as u64)),
        comment: None,
    })?;

    // bit_rate_scale                                    u(4)
    w.fixed_width_field(&FixedWidthField {
        name: "bit_rate_scale",
        bits: 4,
        descriptor: "u(4)",
        value: Some(Value::Unsigned(hrd.bit_rate_scale as u64)),
        comment: None,
    })?;

    // cpb_size_scale                                    u(4)
    w.fixed_width_field(&FixedWidthField {
        name: "cpb_size_scale",
        bits: 4,
        descriptor: "u(4)",
        value: Some(Value::Unsigned(hrd.cpb_size_scale as u64)),
        comment: None,
    })?;

    // for (SchedSelIdx = 0; SchedSelIdx <= cpb_cnt_minus1; SchedSelIdx++)
    w.begin_for(
        "SchedSelIdx = 0; SchedSelIdx <= cpb_cnt_minus1; SchedSelIdx++",
        &[TermAnnotation {
            name: "cpb_cnt_minus1",
            value: Value::Unsigned(cpb_cnt_minus1 as u64),
        }],
    )?;
    for (i, cpb) in hrd.cpb_specs.iter().enumerate() {
        w.for_iteration("SchedSelIdx", i as u64)?;

        w.variable_length_field(&VariableLengthField {
            name: &format!("bit_rate_value_minus1[{i}]"),
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(cpb.bit_rate_value_minus1 as u64)),
            comment: None,
        })?;
        w.variable_length_field(&VariableLengthField {
            name: &format!("cpb_size_value_minus1[{i}]"),
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(cpb.cpb_size_value_minus1 as u64)),
            comment: None,
        })?;
        w.fixed_width_field(&FixedWidthField {
            name: &format!("cbr_flag[{i}]"),
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(cpb.cbr_flag)),
            comment: None,
        })?;
    }
    w.end_for()?;

    // initial_cpb_removal_delay_length_minus1           u(5)
    w.fixed_width_field(&FixedWidthField {
        name: "initial_cpb_removal_delay_length_minus1",
        bits: 5,
        descriptor: "u(5)",
        value: Some(Value::Unsigned(
            hrd.initial_cpb_removal_delay_length_minus1 as u64,
        )),
        comment: None,
    })?;

    // cpb_removal_delay_length_minus1                   u(5)
    w.fixed_width_field(&FixedWidthField {
        name: "cpb_removal_delay_length_minus1",
        bits: 5,
        descriptor: "u(5)",
        value: Some(Value::Unsigned(hrd.cpb_removal_delay_length_minus1 as u64)),
        comment: None,
    })?;

    // dpb_output_delay_length_minus1                    u(5)
    w.fixed_width_field(&FixedWidthField {
        name: "dpb_output_delay_length_minus1",
        bits: 5,
        descriptor: "u(5)",
        value: Some(Value::Unsigned(hrd.dpb_output_delay_length_minus1 as u64)),
        comment: None,
    })?;

    // time_offset_length                                u(5)
    w.fixed_width_field(&FixedWidthField {
        name: "time_offset_length",
        bits: 5,
        descriptor: "u(5)",
        value: Some(Value::Unsigned(hrd.time_offset_length as u64)),
        comment: None,
    })?;

    w.end_element()
}
