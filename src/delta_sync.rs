#[derive(Debug, Clone)]
pub struct DeltaValue<T> {
    pub index: u8,   
    pub data: T,     
}

pub struct SliceDeltaSync;

impl SliceDeltaSync {
    pub fn compare<'a, T: PartialEq + Clone>(old: &'a [T], new: &'a [T]) -> Vec<DeltaValue<T>> {
        let mut changes = Vec::new();
        
        for (i, (old_val, new_val)) in old.iter().zip(new.iter()).enumerate() {
            if old_val != new_val {
                changes.push(DeltaValue {
                    index: i as u8,
                    data: new_val.clone(),
                });
            }
        }
        changes
    }

    pub fn apply<T: Clone>(base: &mut [T], deltas: Vec<DeltaValue<T>>) {
        for delta in deltas {
            if (delta.index as usize) < base.len() {
                base[delta.index as usize] = delta.data;
            }
        }
    }
}

ftest::test!(slice_delta_sync_tests, {
    test_compare_identifies_changes {
        let old = [10, 20, 30, 40];
        let new = [10, 99, 30, 88];

        let deltas = SliceDeltaSync::compare(&old, &new);

        assert_eq!(deltas.len(), 2);
        assert_eq!(deltas[0].index, 1);
        assert_eq!(deltas[0].data, 99);
        assert_eq!(deltas[1].index, 3);
        assert_eq!(deltas[1].data, 88);
    }

    test_apply_updates_base_slice {
        let mut base = [10, 20, 30, 40];
        let deltas = vec![
            DeltaValue { index: 1, data: 99 },
            DeltaValue { index: 3, data: 88 },
        ];

        SliceDeltaSync::apply(&mut base, deltas);

        assert_eq!(base, [10, 99, 30, 88]);
    }

    test_apply_ignores_out_of_bounds {
        let mut base = [10, 20];
        let deltas = vec![
            DeltaValue { index: 0, data: 99 },
            DeltaValue { index: 5, data: 88 },
        ];

        SliceDeltaSync::apply(&mut base, deltas);

        assert_eq!(base, [99, 20]);
    }
});