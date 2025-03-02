use cavernborn::particle::Common;
use strum::IntoEnumIterator;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test to ensure all Common particle variants have exclusive depth ranges
    #[test]
    fn test_common_variants_have_exclusive_ranges() {
        // Get all pairs of Common variants
        let variants: Vec<Common> = Common::iter().collect();

        for (i, variant1) in variants.iter().enumerate() {
            for variant2 in variants.iter().skip(i + 1) {
                // Get the depth ranges for both variants
                let min1 = variant1.min_depth();
                let max1 = variant1.max_depth();
                let min2 = variant2.min_depth();
                let max2 = variant2.max_depth();

                // Check if the ranges overlap using half-open intervals [min, max)
                // Range 1: [min1, max1) and Range 2: [min2, max2)
                // They overlap if: min1 < max2 && min2 < max1
                let overlap = min1 < max2 && min2 < max1;

                // Assert that there is no overlap
                assert!(
                    !overlap,
                    "Common variants {:?} and {:?} have overlapping depth ranges: [{}-{}) and [{}-{})",
                    variant1, variant2, min1, max1, min2, max2
                );
            }
        }
    }

    /// Test to ensure get_exclusive_at_depth returns the correct variant for each depth
    #[test]
    fn test_get_exclusive_at_depth() {
        // Test each Common variant's range
        for variant in Common::iter() {
            let min_depth = variant.min_depth();
            let max_depth = variant.max_depth();

            // Test at the minimum depth (inclusive)
            assert_eq!(
                Common::get_exclusive_at_depth(min_depth),
                variant,
                "get_exclusive_at_depth({}) should return {:?}",
                min_depth,
                variant
            );

            // Test at the maximum depth minus 1 (since max is exclusive)
            if max_depth > min_depth + 1 {
                assert_eq!(
                    Common::get_exclusive_at_depth(max_depth - 1),
                    variant,
                    "get_exclusive_at_depth({}) should return {:?}",
                    max_depth - 1,
                    variant
                );
            }

            // Test at the middle of the range
            if max_depth > min_depth + 2 {
                let mid_depth = min_depth + (max_depth - min_depth) / 2;
                assert_eq!(
                    Common::get_exclusive_at_depth(mid_depth),
                    variant,
                    "get_exclusive_at_depth({}) should return {:?}",
                    mid_depth,
                    variant
                );
            }
        }
    }
}
