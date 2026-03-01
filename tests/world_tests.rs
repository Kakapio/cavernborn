use cavernborn::particle::Common;
use strum::IntoEnumIterator;

#[test]
fn test_common_variants_have_exclusive_ranges() {
    let variants: Vec<Common> = Common::iter().collect();

    for (i, variant1) in variants.iter().enumerate() {
        for variant2 in variants.iter().skip(i + 1) {
            let min1 = variant1.min_depth();
            let max1 = variant1.max_depth();
            let min2 = variant2.min_depth();
            let max2 = variant2.max_depth();

            let overlap = min1 < max2 && min2 < max1;

            assert!(
                !overlap,
                "Common variants {:?} and {:?} have overlapping depth ranges: [{}-{}) and [{}-{})",
                variant1, variant2, min1, max1, min2, max2
            );
        }
    }
}

#[test]
fn test_get_exclusive_at_depth() {
    for variant in Common::iter() {
        let min_depth = variant.min_depth();
        let max_depth = variant.max_depth();

        assert_eq!(
            Common::get_exclusive_at_depth(min_depth),
            variant,
            "get_exclusive_at_depth({}) should return {:?}",
            min_depth,
            variant
        );

        if max_depth > min_depth + 1 {
            assert_eq!(
                Common::get_exclusive_at_depth(max_depth - 1),
                variant,
                "get_exclusive_at_depth({}) should return {:?}",
                max_depth - 1,
                variant
            );
        }

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
