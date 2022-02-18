use core::fmt::Debug;


type UpdateWithOf<T> = <<T as SegNode>::Update as NodeUpdate<T>>::UpdateWith;

pub unsafe trait SegNode: Sized {
    type Merge: NodeMerge<Self>;
    type Update: NodeUpdate<Self>;
}

pub unsafe trait NodeMerge<T> {
    fn merge(n1: &T, n2: &T) -> T;
}

pub unsafe trait NodeUpdate<T> {
    type UpdateWith;

    fn update(node_mut: &mut T, val: &Self::UpdateWith);
}

unsafe impl<T: SegNode + Clone> SegNode for Option<T> {
    type Merge = OptionNodeMerge;
    type Update = OptionNodeUpdate;
}

pub struct OptionNodeMerge;
unsafe impl<T: SegNode + Clone> NodeMerge<Option<T>> for OptionNodeMerge {
    fn merge(n1: &Option<T>, n2: &Option<T>) -> Option<T> {
        match (n1, n2) {
            (Some(n1), Some(n2)) => Some(<<T as SegNode>::Merge as NodeMerge<T>>::merge(n1, n2)),
            (Some(n1), None) => Some(n1.clone()),
            (None, Some(n2)) => Some(n2.clone()),
            _ => None,
        }
    }
}

pub struct OptionNodeUpdate;
unsafe impl<T: SegNode + Clone> NodeUpdate<Option<T>> for OptionNodeUpdate {
    type UpdateWith = UpdateWithOf<T>;

    fn update(node_mut: &mut Option<T>, val: &Self::UpdateWith) {
        match node_mut {
            Some(node_mut) => <<T as SegNode>::Update as NodeUpdate<T>>::update(node_mut, val),
            None => {},
        }
    }
}

pub struct SegmentTree<T: SegNode> {
    tree: Vec<Option<T>>,
    leaf_count: usize,
}

impl<T: SegNode + Clone> SegmentTree<T>{
    pub fn build(data: Vec<T>) -> Self {
        let leaf_count = data.len();
        let tree = vec![None; 4 * leaf_count];

        let mut segment_tree = SegmentTree {
            tree,
            leaf_count,
        };
        unsafe {
            segment_tree.build_internal(&data, 1, 1, segment_tree.leaf_count);
        }
        segment_tree
    }

    pub fn update(&mut self, ind: usize, val: &UpdateWithOf<T>) {
        unsafe {
            self.update_internal(1, 1, self.leaf_count, ind+1 /* 1-indexed */, val);
        }
    }

    pub fn query(&self, lq: usize, rq: usize) -> T {
        unsafe {
            self.query_internal(1, 1, self.leaf_count, lq+1, rq+1).unwrap()
        }
    }

    // 1-indexed
    unsafe fn build_internal(&mut self, data: &[T], ind: usize, l: usize, r: usize) -> bool {
        if self.leaf_count <= l-1 { // self.tree[ind] is a non_cell
            return false;
        }
        
        if l == r {
            //println!("lf, ind, l-1: {}, {}, {}", self.leaf_count, ind, l-1);
            self.tree[ind] = Some(data[l - 1].clone());
            return true;
        }

        let mid = l + (r - l)/2;

        let left_is_cell = self.build_internal(data, 2*ind, l, mid);
        let right_is_cell = self.build_internal(data, 2*ind + 1, mid + 1, r);

        /*match (left_is_cell, right_is_cell) {
            (true, true) => self.tree[ind] = Self::node_merge(self.tree[2*ind].as_ref().unwrap(), &self.tree[2*ind + 1].as_ref().unwrap()),
            (true, false) => self.tree[ind] = self.tree[2*ind].clone(),
            _ => {},
            /*(false, true) => self.tree[ind] = self.tree[2*ind + 1].clone(), // unreachable => by design
            (false, false) => {}, // unreachable => self is is_cell */
        }*/

        self.tree[ind] = Self::node_merge(&self.tree[2*ind], &self.tree[2*ind + 1]);

        true
    }

    unsafe fn update_internal(&mut self, ind: usize, l: usize, r: usize, ind_upd: usize, val: &UpdateWithOf<T>) -> bool {
        if l == r /* == ind_upd */ {
            Self::node_update(self.tree[ind].as_mut().unwrap(), val);
            return true;
        }

        let mid = l + (r - l)/2;

        if ind_upd <= mid {
            self.update_internal(2*ind, l, mid, ind_upd, val);
        }
        else {
            self.update_internal(2*ind + 1, mid+1, r, ind_upd, val);
        }

        /*let left_is_cell = true;
        let right_is_cell = self.leaf_count <= mid /* mid+1 - 1 */;
        match (left_is_cell, right_is_cell) {
            (true, true) => self.tree[ind] = Self::node_merge(self.tree[2*ind].as_ref().unwrap(), self.tree[2*ind + 1].as_ref().unwrap()),
            (true, false) => self.tree[ind] = self.tree[2*ind].clone(),
            _ => {},
            /*(false, true) => self.tree[ind] = self.tree[2*ind + 1].clone(), // unreachable => by design
            (false, false) => {}, // unreachable => self is is_cell */
        }*/

        self.tree[ind] = Self::node_merge(&self.tree[2*ind], &self.tree[2*ind + 1]);

        true
    }

    unsafe fn query_internal(&self, ind: usize, l: usize, r: usize, lq: usize, rq: usize) -> Option<T> {
        if rq < l || r < lq /*|| (self.leaf_count <= l-1) /* already covered by (rq < l) */ */ {
            return None;
        }

        if lq <= l && r <= rq {
            return self.tree[ind].clone();
        }

        let mid = l + (r - l)/2;

        let lval = self.query_internal(2*ind, l, mid, lq, rq);
        let rval = self.query_internal(2*ind + 1, mid+1, r, lq, rq);

        /*match (lval, rval) {
            (Some(lval), Some(rval)) => Self::node_merge(&lval, &rval),
            (Some(lval), None) => Some(lval),
            (None, Some(rval)) => Some(rval),
            _ => None,
        }*/

        Self::node_merge(&lval, &rval)
    }

    fn node_merge<T2: SegNode>(n1: &T2, n2: &T2) -> T2 {
        <<T2 as SegNode>::Merge as NodeMerge<T2>>::merge(n1, n2)
    }

    fn node_update<T2: SegNode>(node_mut: &mut T2, val: &UpdateWithOf<T2>) {
        <<T2 as SegNode>::Update as NodeUpdate<T2>>::update(node_mut, val)
    }
}

impl<T: SegNode + Debug> Debug for SegmentTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SegmentTree").field("tree", &self.tree).field("leaf_count", &self.leaf_count).finish()
    }
}


#[cfg(test)]
mod tests {
    use crate::{SegNode, NodeMerge, NodeUpdate, SegmentTree};

    unsafe impl SegNode for u64 {
        type Merge = AddU64;
        type Update = AddAssignU64;
    }

    #[derive(Debug)]
    pub struct AddU64;
    unsafe impl NodeMerge<u64> for AddU64 {
        fn merge(n1: &u64, n2: &u64) -> u64 {
            n1 + n2
        }
    }

    pub struct AddAssignU64;
    unsafe impl NodeUpdate<u64> for AddAssignU64 {
        type UpdateWith = u64;

        fn update(node_mut: &mut u64, val: &Self::UpdateWith) {
            *node_mut = *node_mut + *val;
        }
    }

    #[test]
    fn it_works() {
        //let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let data = vec![41, 67, 6, 30, 85, 43, 39];
        let mut seg_tree = SegmentTree::build(data);

        let q0_2 = seg_tree.query(0, 2);
        println!("q0_2: {}", q0_2);

        //seg_tree.update(1, &10);

        let q0_2 = seg_tree.query(0, 2);
        println!("q0_2: {}", q0_2);

        println!("{:?}", seg_tree);

        for i in 5..10 {
            println!("0..{}: {}", i, seg_tree.query(5, i));
        }
    }
}
