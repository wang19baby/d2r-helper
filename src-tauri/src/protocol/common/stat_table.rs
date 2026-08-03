//! StatProp 与 StatTable — 从 `MAGICAL_PROPS` 加载或从 itemstatcost.txt 加载。
//!
//! 使用 `[StatProp; 512]` 固定数组替代原 `Vec<StatProp>`，查询为直接索引访问。

pub const MAX_STAT_ID: usize = 511;

#[derive(Debug, Clone, Copy)]
#[derive(Default)]
pub struct StatProp {
    pub save_bits: u8,
    pub num_sub_props: u8,
    pub save_add: i32,
    pub save_param_bits: u8,
    pub signed: u8,
    pub encoding: u8,
    pub descfunc: u8,
    pub cs_bits: u8,
}


impl StatProp {
    pub const fn empty() -> Self {
        Self { save_bits: 0, num_sub_props: 0, save_add: 0, save_param_bits: 0,
               signed: 0, encoding: 0, descfunc: 0, cs_bits: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct StatTable {
    props: [StatProp; MAX_STAT_ID + 1],
}

impl StatTable {
    pub fn from_props(props: Vec<StatProp>) -> Self {
        let mut table = Self::empty();
        let len = props.len().min(MAX_STAT_ID + 1);
        table.props[..len].copy_from_slice(&props[..len]);
        table
    }

    pub fn empty() -> Self {
        Self { props: [StatProp::empty(); MAX_STAT_ID + 1] }
    }

    #[inline]
    pub fn get(&self, id: u16) -> StatProp {
        let idx = id as usize;
        if idx <= MAX_STAT_ID { self.props[idx] } else { StatProp::default() }
    }

    pub fn len(&self) -> usize { MAX_STAT_ID + 1 }
    pub fn is_empty(&self) -> bool { self.props.iter().all(|p| p.save_bits == 0 && p.save_param_bits == 0 && p.save_add == 0) }

    pub fn set(&mut self, id: usize, prop: StatProp) {
        if id <= MAX_STAT_ID { self.props[id] = prop; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_unknown_returns_default() {
        let table = StatTable::empty();
        assert_eq!(table.get(999).save_bits, 0);
    }

    #[test]
    fn test_set_and_get() {
        let mut table = StatTable::empty();
        table.set(42, StatProp { save_bits: 5, ..StatProp::empty() });
        assert_eq!(table.get(42).save_bits, 5);
    }

    #[test]
    fn test_from_props_truncates() {
        let mut v = vec![StatProp::empty(); 600];
        v[0] = StatProp { save_bits: 3, ..StatProp::empty() };
        let table = StatTable::from_props(v);
        assert_eq!(table.get(0).save_bits, 3);
    }
}