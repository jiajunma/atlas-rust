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
#[derive(Clone, Debug)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<i32>,
    trailing_newline: bool,
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows && self.cols == other.cols && self.data == other.data
    }
}

impl Eq for Matrix {}

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
        (data.len() == rows.checked_mul(cols)?).then_some(Self {
            rows,
            cols,
            data,
            trailing_newline: true,
        })
    }

    /// Mark a derived slice for Atlas's compact top-level matrix rendering.
    /// The flag is display-only and is intentionally ignored by equality.
    pub fn without_trailing_newline(mut self) -> Self {
        self.trailing_newline = false;
        self
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

    /// Extract row `row` as a `vec` (upstream `Matrix::row`, global.w:3638).
    pub fn row(&self, row: usize) -> Vec32 {
        assert!(row < self.rows, "matrix row index in range");
        Vec32(
            (0..self.cols)
                .map(|col| self.data[col * self.rows + row])
                .collect(),
        )
    }

    /// Extract column `col` as a `vec` (upstream `Matrix::column`, global.w:3647).
    pub fn column(&self, col: usize) -> Vec32 {
        assert!(col < self.cols, "matrix column index in range");
        Vec32(self.data[col * self.rows..(col + 1) * self.rows].to_vec())
    }

    /// Replace column `col`; the caller checks the size first (the upstream
    /// "Cannot replace column of size R by one of size S" diagnostic).
    pub fn set_column(&mut self, col: usize, column: Vec32) {
        assert!(col < self.cols, "matrix column index in range");
        assert_eq!(column.0.len(), self.rows, "replacement column size matches");
        self.data[col * self.rows..(col + 1) * self.rows].copy_from_slice(&column.0);
    }

    /// Set the entry at (`row`, `col`) (upstream matrix entry assignment).
    pub fn set_entry(&mut self, row: usize, col: usize, value: i32) {
        assert!(row < self.rows && col < self.cols, "matrix entry in range");
        self.data[col * self.rows + row] = value;
    }

    /// `id_mat(n)` (global.w:5190): the `size`×`size` identity matrix.
    pub fn identity(size: usize) -> Self {
        Self::diagonal(&Vec32(vec![1; size]))
    }

    /// Whether every entry is zero (upstream `Matrix_base::is_zero`).
    pub fn is_zero(&self) -> bool {
        self.data.iter().all(|&entry| entry == 0)
    }

    /// `diagonal(v)` (global.w:5191): the square matrix with `entries` on the
    /// diagonal and zeros elsewhere.
    pub fn diagonal(entries: &Vec32) -> Self {
        let size = entries.0.len();
        let data = (0..size)
            .flat_map(|col| (0..size).map(move |row| if row == col { entries.0[col] } else { 0 }))
            .collect();
        Self::from_columns(size, size, data).expect("diagonal data matches size squared")
    }

    /// `^M` (global.w:5186): the transposed matrix.
    pub fn transposed(&self) -> Self {
        let data = (0..self.rows).flat_map(|row| self.row(row).0).collect();
        Self::from_columns(self.cols, self.rows, data)
            .expect("transposed data matches swapped dimensions")
    }

    /// Entry-wise negation; entries wrap like upstream machine `int`.
    pub fn negated(&self) -> Self {
        let data = self
            .data
            .iter()
            .map(|&entry| entry.wrapping_neg())
            .collect();
        Self::from_columns(self.rows, self.cols, data).expect("negation preserves dimensions")
    }

    /// `M += i` (global.w:4235-4248): add `value` to the main diagonal, up to
    /// the smaller dimension (upstream does not require a square matrix).
    pub fn added_diagonal(&self, value: i32) -> Self {
        let mut data = self.data.clone();
        for index in 0..self.rows.min(self.cols) {
            data[index * self.rows + index] = data[index * self.rows + index].wrapping_add(value);
        }
        Self::from_columns(self.rows, self.cols, data).expect("diagonal add preserves dimensions")
    }

    /// `A+B` entry-wise (global.w:4253); the caller checks the shapes match.
    pub fn added(&self, other: &Matrix) -> Self {
        assert_eq!(
            (self.rows, self.cols),
            (other.rows, other.cols),
            "matrix addition sees equal shapes"
        );
        let data = self
            .data
            .iter()
            .zip(&other.data)
            .map(|(&left, &right)| left.wrapping_add(right))
            .collect();
        Self::from_columns(self.rows, self.cols, data).expect("addition preserves dimensions")
    }

    /// `A-B` entry-wise (global.w:4264); the caller checks the shapes match.
    pub fn subtracted(&self, other: &Matrix) -> Self {
        self.added(&other.negated())
    }

    /// `A*B` (global.w:4287); the caller checks `self.cols == other.rows`.
    /// Entries accumulate with machine-`int` wrapping, as upstream.
    pub fn multiplied(&self, other: &Matrix) -> Self {
        assert_eq!(
            self.cols, other.rows,
            "matrix product sees matching inner dimension"
        );
        let data = (0..other.cols)
            .flat_map(|col| {
                (0..self.rows).map(move |row| {
                    let mut sum = 0i32;
                    for inner in 0..self.cols {
                        sum = sum.wrapping_add(
                            self.data[inner * self.rows + row]
                                .wrapping_mul(other.data[col * other.rows + inner]),
                        );
                    }
                    sum
                })
            })
            .collect();
        Self::from_columns(self.rows, other.cols, data)
            .expect("product data matches result dimensions")
    }

    /// `M*v` (global.w:4297); the caller checks `self.cols == vector.len()`.
    pub fn multiplied_vec(&self, vector: &Vec32) -> Vec32 {
        assert_eq!(
            self.cols,
            vector.0.len(),
            "matrix-vector product sees matching dimension"
        );
        Vec32(
            (0..self.rows)
                .map(|row| {
                    let mut sum = 0i32;
                    for (inner, &entry) in vector.0.iter().enumerate() {
                        sum = sum
                            .wrapping_add(self.data[inner * self.rows + row].wrapping_mul(entry));
                    }
                    sum
                })
                .collect(),
        )
    }

    /// `v*M` (global.w:4322); the caller checks `vector.len() == self.rows`.
    pub fn left_multiplied_vec(&self, vector: &Vec32) -> Vec32 {
        assert_eq!(
            self.rows,
            vector.0.len(),
            "vector-matrix product sees matching dimension"
        );
        Vec32(
            (0..self.cols)
                .map(|col| {
                    let mut sum = 0i32;
                    for (row, &entry) in vector.0.iter().enumerate() {
                        sum =
                            sum.wrapping_add(entry.wrapping_mul(self.data[col * self.rows + row]));
                    }
                    sum
                })
                .collect(),
        )
    }

    /// `M*rv` (global.w:4308); the caller checks `self.cols == vector` size.
    /// Numerators accumulate in machine `long` (i64), as upstream.
    pub fn multiplied_ratvec(&self, vector: &RatVec) -> RatVec {
        assert_eq!(
            self.cols,
            vector.numerators().len(),
            "matrix-ratvec product sees matching dimension"
        );
        let numerators = (0..self.rows)
            .map(|row| {
                let mut sum = 0i64;
                for (inner, &numerator) in vector.numerators().iter().enumerate() {
                    sum = sum.wrapping_add(
                        i64::from(self.data[inner * self.rows + row]).wrapping_mul(numerator),
                    );
                }
                sum
            })
            .collect();
        RatVec::new(numerators, vector.denominator()).expect("ratvec denominator stays nonzero")
    }

    /// `rv*M` (global.w:4335); the caller checks `vector` size == self.rows.
    pub fn left_multiplied_ratvec(&self, vector: &RatVec) -> RatVec {
        assert_eq!(
            self.rows,
            vector.numerators().len(),
            "ratvec-matrix product sees matching dimension"
        );
        let numerators = (0..self.cols)
            .map(|col| {
                let mut sum = 0i64;
                for (row, &numerator) in vector.numerators().iter().enumerate() {
                    sum = sum.wrapping_add(
                        numerator.wrapping_mul(i64::from(self.data[col * self.rows + row])),
                    );
                }
                sum
            })
            .collect();
        RatVec::new(numerators, vector.denominator()).expect("ratvec denominator stays nonzero")
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
        let widths: Vec<usize> = self
            .data
            .chunks_exact(self.rows)
            .map(|column| {
                column
                    .iter()
                    .map(|entry| entry.to_string().len())
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        writeln!(formatter)?;
        for row in 0..self.rows {
            write!(formatter, "|")?;
            for (col, column_width) in widths.iter().enumerate() {
                if col > 0 {
                    write!(formatter, ",")?;
                }
                let width = column_width + 1;
                let entry = self.data[col * self.rows + row];
                write!(formatter, "{entry:>width$}")?;
            }
            if row + 1 == self.rows && !self.trailing_newline {
                write!(formatter, " |")?;
            } else {
                writeln!(formatter, " |")?;
            }
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
