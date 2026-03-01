use mpeg_syntax_dump::{FixedWidthField, SyntaxDescribe, SyntaxWrite, Value};

use crate::AudDescribe;

impl SyntaxDescribe for AudDescribe<'_> {
    fn describe<W: SyntaxWrite>(&self, w: &mut W) -> Result<(), W::Error> {
        let aud = self.0;
        w.begin_element("access_unit_delimiter_rbsp", None)?;

        // primary_pic_type                                  u(3)
        w.fixed_width_field(&FixedWidthField {
            name: "primary_pic_type",
            bits: 3,
            descriptor: "u(3)",
            value: Some(Value::Unsigned(aud.primary_pic_type.id() as u64)),
            comment: None,
        })?;

        w.end_element()
    }
}
