use crate::*;

/// An error aggregate for common errors in rust
#[derive(Debug)]
pub struct AbsIndices {
    pub i: usize,
    pub j: usize,
}

/// Miscelaneous functions
pub mod misc {
    use super::*;

    /// Convert indices to positive notation, move them within bounds or return an error
    /// if mutually exclusive.
    pub fn abs_indices(len: isize, i: isize, j: isize) -> Result<AbsIndices> {
        let (mut x, mut y) = (i, j);

        // Convert to postive notation
        if i < 0 {
            x = len + i;
        }
        if j < 0 {
            y = len + j;
        }

        // Start can't be past end else invalid
        if x > y {
            return Err(IterError::mutually_exclusive_indices());
        }

        // Move start/end within bounds
        if x < 0 {
            x = 0
        }
        if y >= len {
            y = len - 1;
        }

        // Rust has an exclusive behavior by default and we want inclusive
        // so offsetting the end by one
        y += 1;

        // return
        Ok(AbsIndices { i: x as usize, j: y as usize })
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_abs_indices() {
        // -4,-3,-2,-1
        //  0, 1, 2, 3

        let tests = vec![
            // single
            ((0, 1), (0, 0)),
            ((1, 2), (1, 1)),
            ((2, 3), (2, 2)),
            ((3, 4), (3, 3)),
            ((3, 4), (-1, -1)),
            ((2, 3), (-2, -2)),
            ((1, 2), (-3, -3)),
            ((0, 1), (-4, -4)),
            // range
            ((0, 4), (-4, -1)),
            ((0, 3), (-4, -2)),
            ((0, 2), (-4, -3)),
            ((0, 1), (-4, -4)),
            // end
            ((1, 4), (-3, -1)),
            ((1, 4), (1, 3)),
            // middle
            ((1, 3), (1, 2)),
            ((1, 3), (-3, -2)),
            // start
            ((0, 3), (0, 2)),
            ((0, 3), (-4, -2)),
            // move within bounds
            ((0, 4), (-5, 5)),
            ((0, 4), (0, 5)),
            ((0, 4), (-5, -1)),
        ];
        for test in tests {
            let val = test.1;
            let exp = test.0;
            let abs = misc::abs_indices(4, val.0, val.1).unwrap();
            assert_eq!(exp.0, abs.i);
            assert_eq!(exp.1, abs.j);
        }

        // mutually exclusive
        assert!(std::panic::catch_unwind(|| misc::abs_indices(4, -1, -3).unwrap()).is_err());
        assert!(std::panic::catch_unwind(|| misc::abs_indices(4, 3, 1).unwrap()).is_err());
    }
}
