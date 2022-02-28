use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use gofund_ust_core::factory_response::AnchorPool;

pub static ANCHOR_POOLS_KEY: &str = "anchor_pool_001";
pub static ANCHOR_POOLS_OWNER_IDX: &str = "anchor_pool_001_owner";
pub static ANCHOR_POOLS_BENEFICIARY_IDX: &str = "anchor_pool_001_beneficiary";
pub static ANCHOR_POOLS_NAME_IDX: &str = "anchor_pool_001_name";

pub struct AnchorPoolIndexes<'a> {
    pub owner: MultiIndex<'a, (String, Vec<u8>), AnchorPool>,
    pub beneficiary: MultiIndex<'a, (String, Vec<u8>), AnchorPool>,
    pub pool: MultiIndex<'a, (String, Vec<u8>), AnchorPool>,
}

impl<'a> IndexList<AnchorPool> for AnchorPoolIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<AnchorPool>> + '_> {
        let v: Vec<&dyn Index<AnchorPool>> = vec![&self.owner, &self.beneficiary, &self.pool];
        Box::new(v.into_iter())
    }
}
pub fn anchor_pool_owner_idx(d: &AnchorPool, k: Vec<u8>) -> (String, Vec<u8>) {
    (d.owner.clone(), k)
}
pub fn anchor_pool_beneficiary_idx(d: &AnchorPool, k: Vec<u8>) -> (String, Vec<u8>) {
    (d.beneficiary.clone(), k)
}
pub fn anchor_pool_name_idx(d: &AnchorPool, k: Vec<u8>) -> (String, Vec<u8>) {
    (d.pool_name.clone(), k)
}

pub fn anchor_pools<'a>() -> IndexedMap<'a, String, AnchorPool, AnchorPoolIndexes<'a>> {
    let indexes = AnchorPoolIndexes {
        owner: MultiIndex::new(
            anchor_pool_owner_idx,
            ANCHOR_POOLS_KEY,
            ANCHOR_POOLS_OWNER_IDX,
        ),
        beneficiary: MultiIndex::new(
            anchor_pool_beneficiary_idx,
            ANCHOR_POOLS_KEY,
            ANCHOR_POOLS_BENEFICIARY_IDX,
        ),
        pool: MultiIndex::new(
            anchor_pool_name_idx,
            ANCHOR_POOLS_KEY,
            ANCHOR_POOLS_NAME_IDX,
        ),
    };
    IndexedMap::new(ANCHOR_POOLS_KEY, indexes)
}
