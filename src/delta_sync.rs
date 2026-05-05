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