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

                // Check if the ranges overlap
                let overlap = (min1 <= max2 && max1 >= min2) || (min2 <= max1 && max2 >= min1);

                // Assert that there is no overlap
                assert!(
                    !overlap,
                    "Common variants {:?} and {:?} have overlapping depth ranges: [{}-{}] and [{}-{}]",
                    variant1, variant2, min1, max1, min2, max2
                );
            }
        }
    }
}
