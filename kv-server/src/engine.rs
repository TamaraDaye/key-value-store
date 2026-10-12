#![allow(unused)]

pub struct MemTable {
}


pub struct BTree {
    root: BTreeNode
}


pub struct BTreeNode<T>{
    keys:  Vec<T>, 
    is_leaf: bool
}


