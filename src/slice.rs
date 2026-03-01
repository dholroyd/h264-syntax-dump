use h264_reader::nal::pps::{PicParameterSet, SliceGroup};
use h264_reader::nal::slice::{
    ColourPlane, DecRefPicMarking, Field, FieldPic, MemoryManagementControlOperation,
    ModificationOfPicNums, NumRefIdxActive, PicOrderCountLsb, PredWeightTable,
    RefPicListModifications, SliceExclusive, SliceFamily,
};
use h264_reader::nal::sps::{ChromaFormat, FrameMbsFlags, PicOrderCntType, SeqParameterSet};
use mpeg_syntax_dump::{
    FixedWidthField, SyntaxDescribe, SyntaxWrite, TermAnnotation, Value, VariableLengthField,
};

use crate::SliceHeaderDescribe;

impl SyntaxDescribe for SliceHeaderDescribe<'_> {
    fn describe<W: SyntaxWrite>(&self, w: &mut W) -> Result<(), W::Error> {
        let hdr = self.header;
        let sps = self.sps;
        let pps = self.pps;
        w.begin_element("slice_header", None)?;

        // first_mb_in_slice                                 ue(v)
        w.variable_length_field(&VariableLengthField {
            name: "first_mb_in_slice",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(hdr.first_mb_in_slice as u64)),
            comment: None,
        })?;

        // slice_type                                        ue(v)
        let slice_type_val = slice_type_to_u32(&hdr.slice_type.family, &hdr.slice_type.exclusive);
        w.variable_length_field(&VariableLengthField {
            name: "slice_type",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(slice_type_val as u64)),
            comment: Some(slice_type_name(&hdr.slice_type.family)),
        })?;

        // pic_parameter_set_id                              ue(v)
        w.variable_length_field(&VariableLengthField {
            name: "pic_parameter_set_id",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(pps.pic_parameter_set_id.id() as u64)),
            comment: None,
        })?;

        // if (separate_colour_plane_flag == 1) colour_plane_id u(2)
        let has_colour_plane = sps.chroma_info.separate_colour_plane_flag;
        w.begin_if(
            "separate_colour_plane_flag == 1",
            &[TermAnnotation {
                name: "separate_colour_plane_flag",
                value: Value::Bool(has_colour_plane),
            }],
            has_colour_plane,
        )?;
        if let Some(cp) = &hdr.colour_plane {
            let id: u8 = match cp {
                ColourPlane::Y => 0,
                ColourPlane::Cb => 1,
                ColourPlane::Cr => 2,
            };
            w.fixed_width_field(&FixedWidthField {
                name: "colour_plane_id",
                bits: 2,
                descriptor: "u(2)",
                value: Some(Value::Unsigned(id as u64)),
                comment: None,
            })?;
        }
        w.end_if()?;

        // frame_num                                         u(v)
        let frame_num_bits = sps.log2_max_frame_num_minus4 as u32 + 4;
        w.fixed_width_field(&FixedWidthField {
            name: "frame_num",
            bits: frame_num_bits,
            descriptor: &format!("u({frame_num_bits})"),
            value: Some(Value::Unsigned(hdr.frame_num as u64)),
            comment: None,
        })?;

        // if (!frame_mbs_only_flag) { field_pic_flag; if (field_pic_flag) bottom_field_flag }
        let not_frame_mbs_only = matches!(sps.frame_mbs_flags, FrameMbsFlags::Fields { .. });
        w.begin_if(
            "!frame_mbs_only_flag",
            &[TermAnnotation {
                name: "frame_mbs_only_flag",
                value: Value::Bool(!not_frame_mbs_only),
            }],
            not_frame_mbs_only,
        )?;
        if not_frame_mbs_only {
            let field_pic_flag = matches!(hdr.field_pic, FieldPic::Field(_));
            w.fixed_width_field(&FixedWidthField {
                name: "field_pic_flag",
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(field_pic_flag)),
                comment: None,
            })?;
            w.begin_if("field_pic_flag", &[], field_pic_flag)?;
            if let FieldPic::Field(field) = &hdr.field_pic {
                let bottom = matches!(field, Field::Bottom);
                w.fixed_width_field(&FixedWidthField {
                    name: "bottom_field_flag",
                    bits: 1,
                    descriptor: "u(1)",
                    value: Some(Value::Bool(bottom)),
                    comment: None,
                })?;
            }
            w.end_if()?;
        }
        w.end_if()?;

        // if (IdrPicFlag) idr_pic_id ue(v)
        let is_idr = hdr.idr_pic_id.is_some();
        w.begin_if(
            "IdrPicFlag",
            &[TermAnnotation {
                name: "IdrPicFlag",
                value: Value::Bool(is_idr),
            }],
            is_idr,
        )?;
        if let Some(id) = hdr.idr_pic_id {
            w.variable_length_field(&VariableLengthField {
                name: "idr_pic_id",
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(id as u64)),
                comment: None,
            })?;
        }
        w.end_if()?;

        // pic_order_cnt fields
        describe_pic_order_cnt_fields(w, hdr, sps, pps)?;

        // if (redundant_pic_cnt_present_flag) redundant_pic_cnt ue(v)
        let has_redundant = pps.redundant_pic_cnt_present_flag;
        w.begin_if(
            "redundant_pic_cnt_present_flag",
            &[TermAnnotation {
                name: "redundant_pic_cnt_present_flag",
                value: Value::Bool(has_redundant),
            }],
            has_redundant,
        )?;
        if let Some(v) = hdr.redundant_pic_cnt {
            w.variable_length_field(&VariableLengthField {
                name: "redundant_pic_cnt",
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(v as u64)),
                comment: None,
            })?;
        }
        w.end_if()?;

        // if (slice_type == B) direct_spatial_mv_pred_flag u(1)
        let is_b = hdr.slice_type.family == SliceFamily::B;
        w.begin_if(
            "slice_type == B",
            &[TermAnnotation {
                name: "slice_type",
                value: Value::Unsigned(slice_type_val as u64),
            }],
            is_b,
        )?;
        if let Some(v) = hdr.direct_spatial_mv_pred_flag {
            w.fixed_width_field(&FixedWidthField {
                name: "direct_spatial_mv_pred_flag",
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(v)),
                comment: None,
            })?;
        }
        w.end_if()?;

        // num_ref_idx_active_override + l0/l1 counts
        let is_p_sp_b = matches!(
            hdr.slice_type.family,
            SliceFamily::P | SliceFamily::SP | SliceFamily::B
        );
        w.begin_if(
            "slice_type == P || slice_type == SP || slice_type == B",
            &[TermAnnotation {
                name: "slice_type",
                value: Value::Unsigned(slice_type_val as u64),
            }],
            is_p_sp_b,
        )?;
        if is_p_sp_b {
            let override_flag = hdr.num_ref_idx_active.is_some();
            w.fixed_width_field(&FixedWidthField {
                name: "num_ref_idx_active_override_flag",
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(override_flag)),
                comment: None,
            })?;
            w.begin_if("num_ref_idx_active_override_flag", &[], override_flag)?;
            if let Some(ref nra) = hdr.num_ref_idx_active {
                match nra {
                    NumRefIdxActive::P {
                        num_ref_idx_l0_active_minus1,
                    } => {
                        w.variable_length_field(&VariableLengthField {
                            name: "num_ref_idx_l0_active_minus1",
                            descriptor: "ue(v)",
                            value: Some(Value::Unsigned(*num_ref_idx_l0_active_minus1 as u64)),
                            comment: None,
                        })?;
                    }
                    NumRefIdxActive::B {
                        num_ref_idx_l0_active_minus1,
                        num_ref_idx_l1_active_minus1,
                    } => {
                        w.variable_length_field(&VariableLengthField {
                            name: "num_ref_idx_l0_active_minus1",
                            descriptor: "ue(v)",
                            value: Some(Value::Unsigned(*num_ref_idx_l0_active_minus1 as u64)),
                            comment: None,
                        })?;
                        w.variable_length_field(&VariableLengthField {
                            name: "num_ref_idx_l1_active_minus1",
                            descriptor: "ue(v)",
                            value: Some(Value::Unsigned(*num_ref_idx_l1_active_minus1 as u64)),
                            comment: None,
                        })?;
                    }
                }
            }
            w.end_if()?;
        }
        w.end_if()?;

        // ref_pic_list_modification()
        if let Some(ref rplm) = hdr.ref_pic_list_modification {
            describe_ref_pic_list_modification(w, rplm, &hdr.slice_type.family, slice_type_val)?;
        }

        // pred_weight_table()
        let need_pwt = (pps.weighted_pred_flag
            && matches!(hdr.slice_type.family, SliceFamily::P | SliceFamily::SP))
            || (pps.weighted_bipred_idc == 1 && hdr.slice_type.family == SliceFamily::B);
        w.begin_if(
            "(weighted_pred_flag && (slice_type == P || slice_type == SP)) || (weighted_bipred_idc == 1 && slice_type == B)",
            &[
                TermAnnotation { name: "weighted_pred_flag", value: Value::Bool(pps.weighted_pred_flag) },
                TermAnnotation { name: "weighted_bipred_idc", value: Value::Unsigned(pps.weighted_bipred_idc as u64) },
                TermAnnotation { name: "slice_type", value: Value::Unsigned(slice_type_val as u64) },
            ],
            need_pwt,
        )?;
        if let Some(ref pwt) = hdr.pred_weight_table {
            describe_pred_weight_table(w, pwt, &hdr.slice_type.family, sps, slice_type_val)?;
        }
        w.end_if()?;

        // dec_ref_pic_marking()
        let has_drpm = hdr.dec_ref_pic_marking.is_some();
        w.begin_if(
            "nal_ref_idc != 0",
            &[TermAnnotation {
                name: "nal_ref_idc",
                value: if has_drpm {
                    Value::Unsigned(1) // exact value unknown, but non-zero
                } else {
                    Value::Unsigned(0)
                },
            }],
            has_drpm,
        )?;
        if let Some(ref drpm) = hdr.dec_ref_pic_marking {
            describe_dec_ref_pic_marking(w, drpm)?;
        }
        w.end_if()?;

        // cabac_init_idc
        let need_cabac = pps.entropy_coding_mode_flag
            && !matches!(hdr.slice_type.family, SliceFamily::I | SliceFamily::SI);
        w.begin_if(
            "entropy_coding_mode_flag && slice_type != I && slice_type != SI",
            &[
                TermAnnotation {
                    name: "entropy_coding_mode_flag",
                    value: Value::Bool(pps.entropy_coding_mode_flag),
                },
                TermAnnotation {
                    name: "slice_type",
                    value: Value::Unsigned(slice_type_val as u64),
                },
            ],
            need_cabac,
        )?;
        if let Some(v) = hdr.cabac_init_idc {
            w.variable_length_field(&VariableLengthField {
                name: "cabac_init_idc",
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(v as u64)),
                comment: None,
            })?;
        }
        w.end_if()?;

        // slice_qp_delta                                    se(v)
        w.variable_length_field(&VariableLengthField {
            name: "slice_qp_delta",
            descriptor: "se(v)",
            value: Some(Value::Signed(hdr.slice_qp_delta as i64)),
            comment: None,
        })?;

        // if (slice_type == SP || slice_type == SI) { sp_for_switch_flag, slice_qs_delta }
        let is_sp_si = matches!(hdr.slice_type.family, SliceFamily::SP | SliceFamily::SI);
        w.begin_if(
            "slice_type == SP || slice_type == SI",
            &[TermAnnotation {
                name: "slice_type",
                value: Value::Unsigned(slice_type_val as u64),
            }],
            is_sp_si,
        )?;
        if is_sp_si {
            let is_sp = hdr.slice_type.family == SliceFamily::SP;
            w.begin_if(
                "slice_type == SP",
                &[TermAnnotation {
                    name: "slice_type",
                    value: Value::Unsigned(slice_type_val as u64),
                }],
                is_sp,
            )?;
            if let Some(v) = hdr.sp_for_switch_flag {
                w.fixed_width_field(&FixedWidthField {
                    name: "sp_for_switch_flag",
                    bits: 1,
                    descriptor: "u(1)",
                    value: Some(Value::Bool(v)),
                    comment: None,
                })?;
            }
            w.end_if()?;

            // h264-reader stores the derived QSY; recover slice_qs_delta
            if let Some(qs_y) = hdr.slice_qs {
                let slice_qs_delta = qs_y as i32 - 26 - pps.pic_init_qs_minus26;
                w.variable_length_field(&VariableLengthField {
                    name: "slice_qs_delta",
                    descriptor: "se(v)",
                    value: Some(Value::Signed(slice_qs_delta as i64)),
                    comment: None,
                })?;
            }
        }
        w.end_if()?;

        // deblocking filter
        let has_deblock = pps.deblocking_filter_control_present_flag;
        w.begin_if(
            "deblocking_filter_control_present_flag",
            &[TermAnnotation {
                name: "deblocking_filter_control_present_flag",
                value: Value::Bool(has_deblock),
            }],
            has_deblock,
        )?;
        if has_deblock {
            w.variable_length_field(&VariableLengthField {
                name: "disable_deblocking_filter_idc",
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(hdr.disable_deblocking_filter_idc as u64)),
                comment: None,
            })?;
            let has_offsets = hdr.disable_deblocking_filter_idc != 1;
            w.begin_if(
                "disable_deblocking_filter_idc != 1",
                &[TermAnnotation {
                    name: "disable_deblocking_filter_idc",
                    value: Value::Unsigned(hdr.disable_deblocking_filter_idc as u64),
                }],
                has_offsets,
            )?;
            if let Some(alpha) = hdr.slice_alpha_c0_offset_div2 {
                w.variable_length_field(&VariableLengthField {
                    name: "slice_alpha_c0_offset_div2",
                    descriptor: "se(v)",
                    value: Some(Value::Signed(alpha as i64)),
                    comment: None,
                })?;
            }
            if let Some(beta) = hdr.slice_beta_offset_div2 {
                w.variable_length_field(&VariableLengthField {
                    name: "slice_beta_offset_div2",
                    descriptor: "se(v)",
                    value: Some(Value::Signed(beta as i64)),
                    comment: None,
                })?;
            }
            w.end_if()?;
        }
        w.end_if()?;

        // slice_group_change_cycle
        if let (
            Some(SliceGroup::Changing {
                slice_group_change_rate_minus1,
                ..
            }),
            Some(cycle),
        ) = (&pps.slice_groups, hdr.slice_group_change_cycle)
        {
            let pic_size = sps.pic_size_in_map_units();
            let change_rate = slice_group_change_rate_minus1 + 1;
            let bits = (f64::from(pic_size) / f64::from(change_rate) + 1.0)
                .log2()
                .ceil() as u32;
            w.fixed_width_field(&FixedWidthField {
                name: "slice_group_change_cycle",
                bits,
                descriptor: &format!("u({bits})"),
                value: Some(Value::Unsigned(cycle as u64)),
                comment: None,
            })?;
        }

        w.end_element()
    }
}

fn slice_type_to_u32(family: &SliceFamily, exclusive: &SliceExclusive) -> u32 {
    let base = match family {
        SliceFamily::P => 0,
        SliceFamily::B => 1,
        SliceFamily::I => 2,
        SliceFamily::SP => 3,
        SliceFamily::SI => 4,
    };
    match exclusive {
        SliceExclusive::NonExclusive => base,
        SliceExclusive::Exclusive => base + 5,
    }
}

fn slice_type_name(family: &SliceFamily) -> &'static str {
    match family {
        SliceFamily::P => "P",
        SliceFamily::B => "B",
        SliceFamily::I => "I",
        SliceFamily::SP => "SP",
        SliceFamily::SI => "SI",
    }
}

fn describe_pic_order_cnt_fields<W: SyntaxWrite>(
    w: &mut W,
    hdr: &h264_reader::nal::slice::SliceHeader,
    sps: &SeqParameterSet,
    pps: &PicParameterSet,
) -> Result<(), W::Error> {
    let poc_type: u32 = match &sps.pic_order_cnt {
        PicOrderCntType::TypeZero { .. } => 0,
        PicOrderCntType::TypeOne { .. } => 1,
        PicOrderCntType::TypeTwo => 2,
    };

    // if (pic_order_cnt_type == 0)
    let is_type0 = poc_type == 0;
    w.begin_if(
        "pic_order_cnt_type == 0",
        &[TermAnnotation {
            name: "pic_order_cnt_type",
            value: Value::Unsigned(poc_type as u64),
        }],
        is_type0,
    )?;
    if let PicOrderCntType::TypeZero {
        log2_max_pic_order_cnt_lsb_minus4,
    } = &sps.pic_order_cnt
    {
        let bits = *log2_max_pic_order_cnt_lsb_minus4 as u32 + 4;
        match &hdr.pic_order_cnt_lsb {
            Some(PicOrderCountLsb::Frame(v)) => {
                w.fixed_width_field(&FixedWidthField {
                    name: "pic_order_cnt_lsb",
                    bits,
                    descriptor: &format!("u({bits})"),
                    value: Some(Value::Unsigned(*v as u64)),
                    comment: None,
                })?;
            }
            Some(PicOrderCountLsb::FieldsAbsolute {
                pic_order_cnt_lsb,
                delta_pic_order_cnt_bottom,
            }) => {
                w.fixed_width_field(&FixedWidthField {
                    name: "pic_order_cnt_lsb",
                    bits,
                    descriptor: &format!("u({bits})"),
                    value: Some(Value::Unsigned(*pic_order_cnt_lsb as u64)),
                    comment: None,
                })?;
                let field_pic_flag = matches!(hdr.field_pic, FieldPic::Field(_));
                w.begin_if(
                    "bottom_field_pic_order_in_frame_present_flag && !field_pic_flag",
                    &[
                        TermAnnotation {
                            name: "bottom_field_pic_order_in_frame_present_flag",
                            value: Value::Bool(pps.bottom_field_pic_order_in_frame_present_flag),
                        },
                        TermAnnotation {
                            name: "field_pic_flag",
                            value: Value::Bool(field_pic_flag),
                        },
                    ],
                    true,
                )?;
                w.variable_length_field(&VariableLengthField {
                    name: "delta_pic_order_cnt_bottom",
                    descriptor: "se(v)",
                    value: Some(Value::Signed(*delta_pic_order_cnt_bottom as i64)),
                    comment: None,
                })?;
                w.end_if()?;
            }
            _ => {}
        }
    }

    // else if (pic_order_cnt_type == 1 && !delta_pic_order_always_zero_flag)
    let delta_pic_order_always_zero_flag = matches!(
        &sps.pic_order_cnt,
        PicOrderCntType::TypeOne {
            delta_pic_order_always_zero_flag: true,
            ..
        }
    );
    let is_type1_non_zero = poc_type == 1 && !delta_pic_order_always_zero_flag;
    w.begin_else_if(
        "pic_order_cnt_type == 1 && !delta_pic_order_always_zero_flag",
        &[TermAnnotation {
            name: "delta_pic_order_always_zero_flag",
            value: Value::Bool(delta_pic_order_always_zero_flag),
        }],
        is_type1_non_zero,
    )?;
    if is_type1_non_zero
        && let Some(PicOrderCountLsb::FieldsDelta(deltas)) = &hdr.pic_order_cnt_lsb {
            w.variable_length_field(&VariableLengthField {
                name: "delta_pic_order_cnt[0]",
                descriptor: "se(v)",
                value: Some(Value::Signed(deltas[0] as i64)),
                comment: None,
            })?;
            if pps.bottom_field_pic_order_in_frame_present_flag && hdr.field_pic == FieldPic::Frame
            {
                let field_pic_flag = matches!(hdr.field_pic, FieldPic::Field(_));
                w.begin_if(
                    "bottom_field_pic_order_in_frame_present_flag && !field_pic_flag",
                    &[
                        TermAnnotation {
                            name: "bottom_field_pic_order_in_frame_present_flag",
                            value: Value::Bool(pps.bottom_field_pic_order_in_frame_present_flag),
                        },
                        TermAnnotation {
                            name: "field_pic_flag",
                            value: Value::Bool(field_pic_flag),
                        },
                    ],
                    true,
                )?;
                w.variable_length_field(&VariableLengthField {
                    name: "delta_pic_order_cnt[1]",
                    descriptor: "se(v)",
                    value: Some(Value::Signed(deltas[1] as i64)),
                    comment: None,
                })?;
                w.end_if()?;
            }
        }
    w.end_if()?;

    Ok(())
}

fn describe_ref_pic_list_modification<W: SyntaxWrite>(
    w: &mut W,
    rplm: &RefPicListModifications,
    family: &SliceFamily,
    slice_type_val: u32,
) -> Result<(), W::Error> {
    w.begin_element("ref_pic_list_modification", None)?;

    let is_not_i_si = !matches!(family, SliceFamily::I | SliceFamily::SI);
    w.begin_if(
        "slice_type % 5 != 2 && slice_type % 5 != 4",
        &[TermAnnotation {
            name: "slice_type",
            value: Value::Unsigned(slice_type_val as u64),
        }],
        is_not_i_si,
    )?;
    if is_not_i_si {
        let (l0_mods, l0_flag) = match rplm {
            RefPicListModifications::P {
                ref_pic_list_modification_l0,
            } => (
                Some(ref_pic_list_modification_l0.as_slice()),
                !ref_pic_list_modification_l0.is_empty(),
            ),
            RefPicListModifications::B {
                ref_pic_list_modification_l0,
                ..
            } => (
                Some(ref_pic_list_modification_l0.as_slice()),
                !ref_pic_list_modification_l0.is_empty(),
            ),
            RefPicListModifications::I => (None, false),
        };

        w.fixed_width_field(&FixedWidthField {
            name: "ref_pic_list_modification_flag_l0",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(l0_flag)),
            comment: None,
        })?;
        w.begin_if("ref_pic_list_modification_flag_l0", &[], l0_flag)?;
        if let Some(mods) = l0_mods
            && l0_flag {
                describe_modification_loop(w, mods)?;
            }
        w.end_if()?;
    }
    w.end_if()?;

    // L1 for B slices
    let is_b = matches!(family, SliceFamily::B);
    w.begin_if(
        "slice_type % 5 == 1",
        &[TermAnnotation {
            name: "slice_type",
            value: Value::Unsigned(slice_type_val as u64),
        }],
        is_b,
    )?;
    if let RefPicListModifications::B {
        ref_pic_list_modification_l1,
        ..
    } = rplm
    {
        let l1_flag = !ref_pic_list_modification_l1.is_empty();
        w.fixed_width_field(&FixedWidthField {
            name: "ref_pic_list_modification_flag_l1",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(l1_flag)),
            comment: None,
        })?;
        w.begin_if("ref_pic_list_modification_flag_l1", &[], l1_flag)?;
        if l1_flag {
            describe_modification_loop(w, ref_pic_list_modification_l1)?;
        }
        w.end_if()?;
    }
    w.end_if()?;

    w.end_element()
}

fn describe_modification_loop<W: SyntaxWrite>(
    w: &mut W,
    mods: &[ModificationOfPicNums],
) -> Result<(), W::Error> {
    w.begin_do_while()?;
    for (i, m) in mods.iter().enumerate() {
        w.do_while_iteration(i as u64)?;
        let (idc, field_name, field_val) = match m {
            ModificationOfPicNums::Subtract(v) => (0u32, "abs_diff_pic_num_minus1", *v),
            ModificationOfPicNums::Add(v) => (1, "abs_diff_pic_num_minus1", *v),
            ModificationOfPicNums::LongTermRef(v) => (2, "long_term_pic_num", *v),
            ModificationOfPicNums::SubtractViewIdx(v) => (4, "abs_diff_view_idx_minus1", *v),
            ModificationOfPicNums::AddViewIdx(v) => (5, "abs_diff_view_idx_minus1", *v),
        };
        w.variable_length_field(&VariableLengthField {
            name: "modification_of_pic_nums_idc",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(idc as u64)),
            comment: None,
        })?;

        let is_01 = idc <= 1;
        w.begin_if(
            "modification_of_pic_nums_idc == 0 || modification_of_pic_nums_idc == 1",
            &[],
            is_01,
        )?;
        if is_01 {
            w.variable_length_field(&VariableLengthField {
                name: "abs_diff_pic_num_minus1",
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(field_val as u64)),
                comment: None,
            })?;
        }
        let is_2 = idc == 2;
        w.begin_else_if("modification_of_pic_nums_idc == 2", &[], is_2)?;
        if is_2 {
            w.variable_length_field(&VariableLengthField {
                name: field_name,
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(field_val as u64)),
                comment: None,
            })?;
        }
        let is_45 = idc == 4 || idc == 5;
        w.begin_else_if(
            "modification_of_pic_nums_idc == 4 || modification_of_pic_nums_idc == 5",
            &[],
            is_45,
        )?;
        if is_45 {
            w.variable_length_field(&VariableLengthField {
                name: "abs_diff_view_idx_minus1",
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(field_val as u64)),
                comment: None,
            })?;
        }
        w.end_if()?;
    }
    // Terminating idc = 3
    w.do_while_iteration(mods.len() as u64)?;
    w.variable_length_field(&VariableLengthField {
        name: "modification_of_pic_nums_idc",
        descriptor: "ue(v)",
        value: Some(Value::Unsigned(3)),
        comment: None,
    })?;
    w.end_do_while("modification_of_pic_nums_idc != 3")
}

fn describe_pred_weight_table<W: SyntaxWrite>(
    w: &mut W,
    pwt: &PredWeightTable,
    family: &SliceFamily,
    sps: &SeqParameterSet,
    slice_type_val: u32,
) -> Result<(), W::Error> {
    w.begin_element("pred_weight_table", None)?;

    let chroma_array_type = if sps.chroma_info.separate_colour_plane_flag {
        ChromaFormat::Monochrome
    } else {
        sps.chroma_info.chroma_format
    };
    let chroma_array_type_val = chroma_array_type.to_u32();
    let has_chroma = chroma_array_type != ChromaFormat::Monochrome;

    // luma_log2_weight_denom                            ue(v)
    w.variable_length_field(&VariableLengthField {
        name: "luma_log2_weight_denom",
        descriptor: "ue(v)",
        value: Some(Value::Unsigned(pwt.luma_log2_weight_denom as u64)),
        comment: None,
    })?;

    // if (ChromaArrayType != 0) chroma_log2_weight_denom ue(v)
    w.begin_if(
        "ChromaArrayType != 0",
        &[TermAnnotation {
            name: "ChromaArrayType",
            value: Value::Unsigned(chroma_array_type_val as u64),
        }],
        has_chroma,
    )?;
    if let Some(v) = pwt.chroma_log2_weight_denom {
        w.variable_length_field(&VariableLengthField {
            name: "chroma_log2_weight_denom",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(v as u64)),
            comment: None,
        })?;
    }
    w.end_if()?;

    // L0 weights
    let num_l0 = pwt.luma_weights.len();
    w.begin_for(
        "i = 0; i <= num_ref_idx_l0_active_minus1; i++",
        &[TermAnnotation {
            name: "num_ref_idx_l0_active_minus1",
            value: Value::Unsigned(num_l0.saturating_sub(1) as u64),
        }],
    )?;
    for (i, lw) in pwt.luma_weights.iter().enumerate() {
        w.for_iteration("i", i as u64)?;
        let has_luma = lw.is_some();
        w.fixed_width_field(&FixedWidthField {
            name: "luma_weight_l0_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(has_luma)),
            comment: None,
        })?;
        w.begin_if("luma_weight_l0_flag", &[], has_luma)?;
        if let Some(pw) = lw {
            w.variable_length_field(&VariableLengthField {
                name: &format!("luma_weight_l0[{i}]"),
                descriptor: "se(v)",
                value: Some(Value::Signed(pw.weight as i64)),
                comment: None,
            })?;
            w.variable_length_field(&VariableLengthField {
                name: &format!("luma_offset_l0[{i}]"),
                descriptor: "se(v)",
                value: Some(Value::Signed(pw.offset as i64)),
                comment: None,
            })?;
        }
        w.end_if()?;

        if has_chroma {
            let cw = pwt.chroma_weights.get(i);
            let has_cw = cw.is_some_and(|v| !v.is_empty());
            w.begin_if(
                "ChromaArrayType != 0",
                &[TermAnnotation {
                    name: "ChromaArrayType",
                    value: Value::Unsigned(chroma_array_type_val as u64),
                }],
                true,
            )?;
            w.fixed_width_field(&FixedWidthField {
                name: "chroma_weight_l0_flag",
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(has_cw)),
                comment: None,
            })?;
            w.begin_if("chroma_weight_l0_flag", &[], has_cw)?;
            if let Some(weights) = cw {
                for (j, pw) in weights.iter().enumerate() {
                    w.variable_length_field(&VariableLengthField {
                        name: &format!("chroma_weight_l0[{i}][{j}]"),
                        descriptor: "se(v)",
                        value: Some(Value::Signed(pw.weight as i64)),
                        comment: None,
                    })?;
                    w.variable_length_field(&VariableLengthField {
                        name: &format!("chroma_offset_l0[{i}][{j}]"),
                        descriptor: "se(v)",
                        value: Some(Value::Signed(pw.offset as i64)),
                        comment: None,
                    })?;
                }
            }
            w.end_if()?;
            w.end_if()?;
        }
    }
    w.end_for()?;

    // L1 weights (B slices only)
    let is_b = matches!(family, SliceFamily::B);
    w.begin_if(
        "slice_type % 5 == 1",
        &[TermAnnotation {
            name: "slice_type",
            value: Value::Unsigned(slice_type_val as u64),
        }],
        is_b,
    )?;
    if is_b && !pwt.luma_weights_l1.is_empty() {
        let num_l1 = pwt.luma_weights_l1.len();
        w.begin_for(
            "i = 0; i <= num_ref_idx_l1_active_minus1; i++",
            &[TermAnnotation {
                name: "num_ref_idx_l1_active_minus1",
                value: Value::Unsigned(num_l1.saturating_sub(1) as u64),
            }],
        )?;
        for (i, lw) in pwt.luma_weights_l1.iter().enumerate() {
            w.for_iteration("i", i as u64)?;
            let has_luma = lw.is_some();
            w.fixed_width_field(&FixedWidthField {
                name: "luma_weight_l1_flag",
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(has_luma)),
                comment: None,
            })?;
            w.begin_if("luma_weight_l1_flag", &[], has_luma)?;
            if let Some(pw) = lw {
                w.variable_length_field(&VariableLengthField {
                    name: &format!("luma_weight_l1[{i}]"),
                    descriptor: "se(v)",
                    value: Some(Value::Signed(pw.weight as i64)),
                    comment: None,
                })?;
                w.variable_length_field(&VariableLengthField {
                    name: &format!("luma_offset_l1[{i}]"),
                    descriptor: "se(v)",
                    value: Some(Value::Signed(pw.offset as i64)),
                    comment: None,
                })?;
            }
            w.end_if()?;

            if has_chroma {
                let cw = pwt.chroma_weights_l1.get(i);
                let has_cw = cw.is_some_and(|v| !v.is_empty());
                w.begin_if(
                    "ChromaArrayType != 0",
                    &[TermAnnotation {
                        name: "ChromaArrayType",
                        value: Value::Unsigned(chroma_array_type_val as u64),
                    }],
                    true,
                )?;
                w.fixed_width_field(&FixedWidthField {
                    name: "chroma_weight_l1_flag",
                    bits: 1,
                    descriptor: "u(1)",
                    value: Some(Value::Bool(has_cw)),
                    comment: None,
                })?;
                w.begin_if("chroma_weight_l1_flag", &[], has_cw)?;
                if let Some(weights) = cw {
                    for (j, pw) in weights.iter().enumerate() {
                        w.variable_length_field(&VariableLengthField {
                            name: &format!("chroma_weight_l1[{i}][{j}]"),
                            descriptor: "se(v)",
                            value: Some(Value::Signed(pw.weight as i64)),
                            comment: None,
                        })?;
                        w.variable_length_field(&VariableLengthField {
                            name: &format!("chroma_offset_l1[{i}][{j}]"),
                            descriptor: "se(v)",
                            value: Some(Value::Signed(pw.offset as i64)),
                            comment: None,
                        })?;
                    }
                }
                w.end_if()?;
                w.end_if()?;
            }
        }
        w.end_for()?;
    }
    w.end_if()?;

    w.end_element()
}

fn describe_dec_ref_pic_marking<W: SyntaxWrite>(
    w: &mut W,
    drpm: &DecRefPicMarking,
) -> Result<(), W::Error> {
    w.begin_element("dec_ref_pic_marking", None)?;

    let is_idr = matches!(drpm, DecRefPicMarking::Idr { .. });
    w.begin_if(
        "IdrPicFlag",
        &[TermAnnotation {
            name: "IdrPicFlag",
            value: Value::Bool(is_idr),
        }],
        is_idr,
    )?;
    if let DecRefPicMarking::Idr {
        no_output_of_prior_pics_flag,
        long_term_reference_flag,
    } = drpm
    {
        w.fixed_width_field(&FixedWidthField {
            name: "no_output_of_prior_pics_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(*no_output_of_prior_pics_flag)),
            comment: None,
        })?;
        w.fixed_width_field(&FixedWidthField {
            name: "long_term_reference_flag",
            bits: 1,
            descriptor: "u(1)",
            value: Some(Value::Bool(*long_term_reference_flag)),
            comment: None,
        })?;
    }
    w.begin_else(!is_idr)?;
    match drpm {
        DecRefPicMarking::SlidingWindow => {
            w.fixed_width_field(&FixedWidthField {
                name: "adaptive_ref_pic_marking_mode_flag",
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(false)),
                comment: None,
            })?;
        }
        DecRefPicMarking::Adaptive(ops) => {
            w.fixed_width_field(&FixedWidthField {
                name: "adaptive_ref_pic_marking_mode_flag",
                bits: 1,
                descriptor: "u(1)",
                value: Some(Value::Bool(true)),
                comment: None,
            })?;
            w.begin_if("adaptive_ref_pic_marking_mode_flag", &[], true)?;
            describe_mmco_loop(w, ops)?;
            w.end_if()?;
        }
        DecRefPicMarking::Idr { .. } => {} // handled above
    }
    w.end_if()?;

    w.end_element()
}

fn describe_mmco_loop<W: SyntaxWrite>(
    w: &mut W,
    ops: &[MemoryManagementControlOperation],
) -> Result<(), W::Error> {
    w.begin_do_while()?;
    for (i, op) in ops.iter().enumerate() {
        w.do_while_iteration(i as u64)?;
        use MemoryManagementControlOperation as Mmco;
        let (op_val, fields): (u32, Vec<(&str, u32)>) = match op {
            Mmco::ShortTermUnusedForRef {
                difference_of_pic_nums_minus1,
            } => (
                1,
                vec![(
                    "difference_of_pic_nums_minus1",
                    *difference_of_pic_nums_minus1,
                )],
            ),
            Mmco::LongTermUnusedForRef { long_term_pic_num } => {
                (2, vec![("long_term_pic_num", *long_term_pic_num)])
            }
            Mmco::ShortTermUsedForLongTerm {
                difference_of_pic_nums_minus1,
                long_term_frame_idx,
            } => (
                3,
                vec![
                    (
                        "difference_of_pic_nums_minus1",
                        *difference_of_pic_nums_minus1,
                    ),
                    ("long_term_frame_idx", *long_term_frame_idx),
                ],
            ),
            Mmco::MaxUsedLongTermFrameRef {
                max_long_term_frame_idx_plus1,
            } => (
                4,
                vec![(
                    "max_long_term_frame_idx_plus1",
                    *max_long_term_frame_idx_plus1,
                )],
            ),
            Mmco::AllRefPicturesUnused => (5, vec![]),
            Mmco::CurrentUsedForLongTerm {
                long_term_frame_idx,
            } => (6, vec![("long_term_frame_idx", *long_term_frame_idx)]),
        };
        w.variable_length_field(&VariableLengthField {
            name: "memory_management_control_operation",
            descriptor: "ue(v)",
            value: Some(Value::Unsigned(op_val as u64)),
            comment: None,
        })?;
        for (name, val) in &fields {
            w.variable_length_field(&VariableLengthField {
                name,
                descriptor: "ue(v)",
                value: Some(Value::Unsigned(*val as u64)),
                comment: None,
            })?;
        }
    }
    // Terminating operation 0
    w.do_while_iteration(ops.len() as u64)?;
    w.variable_length_field(&VariableLengthField {
        name: "memory_management_control_operation",
        descriptor: "ue(v)",
        value: Some(Value::Unsigned(0)),
        comment: None,
    })?;
    w.end_do_while("memory_management_control_operation != 0")
}
