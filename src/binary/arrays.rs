use std::io::{self, Write};

use super::{BinaryRead, BinaryReadTracked, BinaryWrite, FieldRange, push_index, pop_path};

impl<'a> BinaryRead<'a> for [f32; 3] {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        Ok([
            f32::read_from(data, offset)?,
            f32::read_from(data, offset)?,
            f32::read_from(data, offset)?,
        ])
    }
}

impl BinaryWrite for [f32; 3] {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        for v in self {
            v.write_to(w)?;
        }
        Ok(())
    }
}

impl<'a> BinaryRead<'a> for [u32; 4] {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        Ok([
            u32::read_from(data, offset)?,
            u32::read_from(data, offset)?,
            u32::read_from(data, offset)?,
            u32::read_from(data, offset)?,
        ])
    }
}

impl BinaryWrite for [u32; 4] {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        for v in self {
            v.write_to(w)?;
        }
        Ok(())
    }
}

// Generic [u8; N] - covers N=3 (Porter's original impl) plus all sizes
// emitted by generated tables/ modules (e.g. NattKh's direct_15B fields).
impl<'a, const N: usize> BinaryRead<'a> for [u8; N] {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        super::check_remaining(data, *offset, N)?;
        let arr: [u8; N] = data[*offset..*offset + N].try_into().unwrap();
        *offset += N;
        Ok(arr)
    }
}

impl<const N: usize> BinaryWrite for [u8; N] {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(self)
    }
}

// ── Fixed-size array tracked reads ──────────────────────────────────────────
// Each element is reported as `<path>[i]` so the byte layout is preserved.

impl<'a> BinaryReadTracked<'a> for [f32; 3] {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let mut out = [0f32; 3];
        for i in 0..3 {
            let saved = push_index(path, i);
            out[i] = f32::read_tracked(data, offset, path, ranges)?;
            pop_path(path, saved);
        }
        Ok(out)
    }
}

impl<'a> BinaryReadTracked<'a> for [u32; 4] {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let mut out = [0u32; 4];
        for i in 0..4 {
            let saved = push_index(path, i);
            out[i] = u32::read_tracked(data, offset, path, ranges)?;
            pop_path(path, saved);
        }
        Ok(out)
    }
}

impl<'a, const N: usize> BinaryReadTracked<'a> for [u8; N] {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let mut out = [0u8; N];
        for i in 0..N {
            let saved = push_index(path, i);
            out[i] = u8::read_tracked(data, offset, path, ranges)?;
            pop_path(path, saved);
        }
        Ok(out)
    }
}
