//! Concrete vec / mat / ratvec payloads with the upstream print formats.
//!
//! Payloads match upstream exactly (axis-types.w:2028-2094): vec entries are
//! MACHINE 32-bit integers, matrices store int entries column-major, ratvec
//! keeps i64 numerators over a u64 denominator normalised on construction.
//! Printing follows global.w:2107-2158 byte for byte — right-aligned width
//! fields, the `" ]"` tail, per-column matrix widths inside `|` frames, and
//! the empty-matrix wording. These embed into `Value` at phase-B stage B2;
//! until then the module stands alone.

use std::fmt;

/// An Atlas `vec`: machine 32-bit entries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vec32(pub Vec<i32>);

/// An Atlas `mat`: `rows x cols` int entries stored column-major (the
/// upstream constructor fills by columns).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<i32>,
}

/// An Atlas `ratvec`: i64 numerators over one u64 denominator, normalised
/// (gcd divided out, denominator positive) on every construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RatVec {
    numerators: Vec<i64>,
    denominator: u64,
}

impl Matrix {
    /// Build from column-major data; `data.len()` must be `rows * cols`.
    pub fn from_columns(rows: usize, cols: usize, data: Vec<i32>) -> Option<Self> {
        (data.len() == rows.checked_mul(cols)?).then_some(Self { rows, cols, data })
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn entry(&self, row: usize, col: usize) -> Option<i32> {
        (row < self.rows && col < self.cols).then(|| self.data[col * self.rows + row])
    }
}

impl RatVec {
    pub fn new(numerators: Vec<i64>, denominator: u64) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        let mut divisor = denominator;
        for &numerator in &numerators {
            divisor = gcd_u64(divisor, numerator.unsigned_abs());
        }
        if divisor <= 1 {
            return Some(Self {
                numerators,
                denominator,
            });
        }
        Some(Self {
            numerators: numerators
                .into_iter()
                .map(|numerator| numerator / divisor as i64)
                .collect(),
            denominator: denominator / divisor,
        })
    }

    pub fn numerators(&self) -> &[i64] {
        &self.numerators
    }

    pub fn denominator(&self) -> u64 {
        self.denominator
    }
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// The shared bracket layout of vec and ratvec numerators: entries
/// right-aligned in width (max entry width + 1), commas between, `" ]"`
/// after the last entry; `"[ ]"` when empty.
fn write_bracketed<T: fmt::Display>(entries: &[T], out: &mut fmt::Formatter<'_>) -> fmt::Result {
    if entries.is_empty() {
        return write!(out, "[ ]");
    }
    let rendered: Vec<String> = entries.iter().map(T::to_string).collect();
    let width = rendered
        .iter()
        .map(String::len)
        .max()
        .expect("nonempty entries")
        + 1;
    write!(out, "[")?;
    for (index, entry) in rendered.iter().enumerate() {
        if index > 0 {
            write!(out, ",")?;
        }
        write!(out, "{entry:>width$}")?;
    }
    write!(out, " ]")
}

impl fmt::Display for Vec32 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_bracketed(&self.0, formatter)
    }
}

impl fmt::Display for RatVec {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_bracketed(&self.numerators, formatter)?;
        write!(formatter, "/{}", self.denominator)
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.rows == 0 || self.cols == 0 {
            return write!(formatter, "The {}x{} matrix", self.rows, self.cols);
        }
        // Per-column widths; entries right-aligned in width[col] + 1.
        let mut widths = vec![0_usize; self.cols];
        for col in 0..self.cols {
            for row in 0..self.rows {
                let rendered = self.data[col * self.rows + row].to_string().len();
                widths[col] = widths[col].max(rendered);
            }
        }
        writeln!(formatter)?;
        for row in 0..self.rows {
            write!(formatter, "|")?;
            for col in 0..self.cols {
                if col > 0 {
                    write!(formatter, ",")?;
                }
                let width = widths[col] + 1;
                let entry = self.data[col * self.rows + row];
                write!(formatter, "{entry:>width$}")?;
            }
            writeln!(formatter, " |")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_prints_right_aligned_with_the_space_bracket_tail() {
        assert_eq!(Vec32(vec![1, 22]).to_string(), "[  1, 22 ]");
        assert_eq!(Vec32(vec![-3]).to_string(), "[ -3 ]");
        assert_eq!(Vec32(Vec::new()).to_string(), "[ ]");
    }

    #[test]
    fn ratvec_normalises_and_prints_numerators_over_the_denominator() {
        let ratvec = RatVec::new(vec![2, 4], 6).expect("valid denominator");
        assert_eq!(ratvec.numerators(), &[1, 2]);
        assert_eq!(ratvec.denominator(), 3);
        assert_eq!(ratvec.to_string(), "[ 1, 2 ]/3");
        assert!(RatVec::new(vec![1], 0).is_none());
        // No common factor: unchanged.
        let raw = RatVec::new(vec![1, 2], 2).expect("valid");
        assert_eq!(raw.to_string(), "[ 1, 2 ]/2");
    }

    #[test]
    fn matrix_prints_column_widths_inside_bar_frames() {
        let matrix = Matrix::from_columns(2, 2, vec![1, 3, 2, 44]).expect("consistent dimensions");
        assert_eq!(matrix.entry(0, 1), Some(2));
        assert_eq!(matrix.to_string(), "\n| 1,  2 |\n| 3, 44 |\n");
        let empty = Matrix::from_columns(0, 3, Vec::new()).expect("empty is consistent");
        assert_eq!(empty.to_string(), "The 0x3 matrix");
    }
}
