// use crate::{trunc32_to_n, HashingAlgorithm, MerkleError, MerkleHash};
// #[cfg(target_os = "solana")]
// use anchor_lang::Result;
// #[cfg(not(target_os = "solana"))]
// use anyhow::Result;
// #[cfg(not(target_os = "solana"))]
// use rayon::{
//     iter::{IntoParallelIterator, ParallelIterator},
//     prelude::*,
// };

// use core::{
//     hash::SipHasher,
//     mem::{transmute, MaybeUninit},
// };

// use alloc::vec;
// use alloc::vec::Vec;

// use super::MerkleProof;

// #[derive(Debug, Clone)]
// pub struct MerkleTree<N: MerkleHash<N>, const N: usize, const NODES: usize> {
//     algorithm: HashingAlgorithm<N>,
//     root: N,
//     nodes: [N; NODES],
// }

// // // For non-Solana targets, use Rayon to hash/merklize in parallel
// // #[cfg(not(target_os = "solana"))]
// // impl<N: MerkleHash<N>, const N: usize, const NODES: usize> MerkleTree<N, N, NODES> {
// //     /// Compute the next layer of the Merkle tree in parallel, stack-allocated.
// //     pub fn merklize_unchecked(h: &[[u8; N]], a: HashingAlgorithm<N>) -> Vec<[u8; N]> {
// //         h.par_chunks(2)
// //             .map(|pair| {
// //                 if pair.len() == 2 {
// //                     a.hash(&[pair[0].as_ref(), pair[1].as_ref()].concat())
// //                 } else {
// //                     // duplicate last nodes if odd number
// //                     a.hash(&[pair[0].as_ref(), pair[0].as_ref()].concat())
// //                 }
// //             })
// //             .collect()
// //     }

// //     pub fn add_leaves(&mut self, leaves: &[[u8; N]]) -> Result<()> {
// //         let nodes: Vec<[u8; N]> = leaves
// //             .into_par_iter()
// //             .map(|leaf| self.double_hash(leaf))
// //             .collect();
// //         // self.add_hashes_unchecked(nodes)

// //         Ok(())
// //     }
// // }

// // For Solana targets, merklize in serial
// // #[cfg(target_os = "solana")]
// impl<N: MerkleHash<N>, const N: usize, const NODES: usize> MerkleTree<N, N, NODES> {
//     fn merklize_unchecked(h: &[[u8; N]], a: HashingAlgorithm<N>, s: usize) -> [[u8; N]; NODES] {
//         let mut out: [[u8; N]; NODES] = [[0u8; N]; NODES];

//         for (i, pair) in h.chunks(2).enumerate() {
//             out[i] = if pair.len() == 2 {
//                 a.hash(&[pair[0].as_ref(), pair[1].as_ref()].concat())
//             } else {
//                 a.hash(&[pair[0].as_ref(), pair[0].as_ref()].concat())
//             };
//         }

//         out
//     }

//     // pub fn add_leaves(&mut self, leaves: &Vec<Vec<u8>>) -> Result<()> {
//     //     let nodes: Vec<Vec<u8>> = leaves
//     //         .into_iter()
//     //         .map(|leaf| self.double_hash(leaf))
//     //         .collect();
//     //     self.add_hashes_unchecked(nodes)
//     // }
// }

// impl<N: MerkleHash<N>, const N: usize, const NODES: usize> MerkleTree<N, N, NODES> {
//     // Initialize a new tree with configurable size and hashing params
//     pub fn new(algorithm: HashingAlgorithm<N>) -> Self {
//         Self {
//             algorithm,
//             root: N::default(),
//             nodes: [N::default(); NODES],
//         }
//     }

//     // Append multiple nodes with a length check. Use with unnormalized data
//     // pub fn add_hashes(&mut self, nodes: Vec<Vec<u8>>) -> Result<()> {
//     //     for hash in nodes.iter() {
//     //         if hash.len() != self.hash_size as usize {
//     //             return Err(MerkleError::InvalidHashSize.into());
//     //         }
//     //     }
//     //     self.nodes[0].extend_from_slice(&nodes);
//     //     Ok(())
//     // }

//     // // Append multiple nodes without a length check. Use with normalized data
//     // pub fn add_hashes_unchecked(&mut self, nodes: &[[u8; N]]) -> Result<()> {
//     //     self.nodes
//     //     Ok(())
//     // }

//     // Double hash with defined hashing algorithm and truncate to defined length
//     fn double_hash(&self, m: &[u8]) -> [u8; N] {
//         unsafe { trunc32_to_n(self.algorithm.double_hash_32(m).as_ptr()) }
//     }

//     // Hash and append a leaf
//     // pub fn add_leaf(&mut self, leaf: &[u8]) {
//     //     // Double hash to prevent length extension attacks
//     //     // No need for length check
//     //     self.add_hash_unchecked(self.double_hash(leaf))
//     // }

//     // Append a hash with a length check. Use with unnormalized data
//     // pub fn add_hash(&mut self, hash: Vec<u8>) -> Result<()> {
//     //     if hash.len() != self.hash_size as usize {
//     //         return Err(MerkleError::InvalidHashSize.into());
//     //     }
//     //     self.add_hash_unchecked(hash);
//     //     Ok(())
//     // }

//     // Append a hash without a length check. Use with normalized data
//     // pub fn add_hash_unchecked(&mut self, hash: Vec<u8>) {
//     //     self.nodes[0].push(hash);
//     // }

//     pub fn merklize(&mut self) -> Result<()> {
//         let len = self.nodes[0].len();
//         match len {
//             0 => Err(MerkleError::TreeEmpty.into()),
//             1 => {
//                 self.reset();
//                 self.root = self.nodes[0][0].clone();
//                 Ok(())
//             }
//             _ => {
//                 self.reset();
//                 let mut count = self.nodes[0].len();
//                 while count > 2 {
//                     let h: Vec<Vec<u8>> = Self::merklize_unchecked(
//                         self.nodes.last().ok_or(MerkleError::BranchOutOfRange)?,
//                         self.algorithm.clone(),
//                         self.hash_size as usize,
//                     );
//                     count = h.len();
//                     self.nodes.push(h);
//                 }
//                 self.root = Self::merklize_unchecked(
//                     self.nodes.last().ok_or(MerkleError::BranchOutOfRange)?,
//                     self.algorithm.clone(),
//                     32_usize,
//                 )[0]
//                 .clone();
//                 Ok(())
//             }
//         }
//     }

//     pub fn reset(&mut self) {
//         self.nodes.truncate(1);
//     }

//     fn merklized(&self) -> Result<()> {
//         if self.root.eq(&[0u8; 32]) {
//             return Err(MerkleError::TreeNotMerklized.into());
//         }
//         Ok(())
//     }

//     fn within_range(&self, index: usize) -> Result<()> {
//         let len = self.nodes[0].len();
//         if index > len {
//             return Err(MerkleError::LeafOutOfRange.into());
//         }
//         Ok(())
//     }

//     fn get_hash_index(&self, hash: Vec<u8>) -> Result<usize> {
//         match self.nodes[0].binary_search(&hash) {
//             Ok(i) => Ok(i),
//             Err(_) => Err(MerkleError::LeafNotFound.into()),
//         }
//     }

//     // pub fn pairing_hashes_hash(&self, hash: Vec<u8>) -> Result<Vec<u8>> {
//     //     let proof = self.merkle_proof_hash(hash)?;
//     //     proof.to_pairing_hashes()
//     // }

//     // pub fn pairing_hashes_index(&self, index: usize) -> Result<Vec<u8>> {
//     //     let proof = self.merkle_proof_index(index)?;
//     //     proof.to_pairing_hashes()
//     // }

//     pub fn get_merkle_root(&self) -> Result<Vec<u8>> {
//         self.merklized()?;
//         Ok(self.root.clone())
//     }

//     pub fn get_leaf_hash(&self, i: usize) -> Result<Vec<u8>> {
//         self.within_range(i)?;
//         Ok(self.nodes[0][i].clone())
//     }

//     pub fn merkle_proof_hash(&self, hash: Vec<u8>) -> Result<MerkleProof> {
//         self.merklized()?;
//         let i = self.get_hash_index(hash)?;
//         self.merkle_proof_index_unchecked(i)
//     }

//     pub fn merkle_proof_index(&self, i: usize) -> Result<MerkleProof> {
//         self.merklized()?;
//         self.within_range(i)?;
//         self.merkle_proof_index_unchecked(i)
//     }

//     fn merkle_proof_index_unchecked(&self, i: usize) -> Result<MerkleProof> {
//         let len = self.nodes[0].len();
//         match len {
//             // We can't have zero leaves in a Merkle tree
//             0 => Err(MerkleError::TreeEmpty.into()),
//             // If we only have one leaf, the 0th hash is the root
//             1 => Ok(MerkleProof::new(
//                 self.algorithm.clone(),
//                 self.hash_size,
//                 i as u32,
//                 vec![],
//             )),
//             _ => {
//                 let mut nodes: Vec<Vec<u8>> = vec![];
//                 let mut n = i;
//                 // 0, 1, 2, 3
//                 for x in 0..self.nodes.len() {
//                     n = match n % 2 == 0 {
//                         true => usize::min(n + 1, self.nodes[x].len()),
//                         false => n - 1,
//                     };

//                     match self.nodes[x].get(n) {
//                         Some(h) => nodes.push(h.clone()),
//                         None => nodes.push(self.nodes[x][n - 1].clone()),
//                     }
//                     n = n.saturating_div(2);
//                 }
//                 Ok(MerkleProof::new(
//                     self.algorithm.clone(),
//                     self.hash_size,
//                     i as u32,
//                     nodes.concat(),
//                 ))
//             }
//         }
//     }
// }

use core::marker::PhantomData;

use crate::{MerkleError, MerkleHash};
#[cfg(not(target_os = "solana"))]
use anyhow::Result;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MerkleNode<const N: usize>([u8; N], bool); // (buffer , is_occupied)

impl<const N: usize> core::fmt::Debug for MerkleNode<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Convert to hex string
        writeln!(f, "occupied : {}", self.1)?;
        write!(f, "hash hex : {}", hex::encode(self.0))
    }
}

impl<const N: usize> MerkleNode<N> {
    pub fn inner(&self) -> &[u8] {
        &self.0
    }

    pub fn is_occupied(&self) -> bool {
        self.1
    }

    pub fn is_not_occupied(&self) -> bool {
        !self.is_occupied() && self.0 == [0; N]
    }

    pub fn inner_cloned(&self) -> [u8; N] {
        self.0.clone()
    }
}

impl<const N: usize> From<[u8; N]> for MerkleNode<N> {
    fn from(bytes: [u8; N]) -> Self {
        Self(bytes, true)
    }
}

impl<const N: usize> Default for MerkleNode<N> {
    fn default() -> Self {
        MerkleNode([0; N], false)
    }
}

// helper to hex-encode a byte array
fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

impl<H: MerkleHash<N>, const N: usize, const NODES: usize> core::fmt::Debug
    for MerkleTree<H, N, NODES>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let root_hex = to_hex(&self.root.0);
        let hashes_hex: Vec<String> = self.nodes.iter().map(|h| to_hex(&h.0)).collect();

        f.debug_struct("MerkleTree")
            .field("root", &root_hex)
            .field("nodes", &hashes_hex)
            .field("leaf_count", &self.leaf_count)
            .finish()
    }
}

pub const fn nodes_from_depth(depth: usize) -> usize {
    (1 << (depth + 1)) - 1
}

#[derive(Clone)]
pub struct MerkleTree<H: MerkleHash<N>, const N: usize, const NODES: usize> {
    pub root: MerkleNode<N>,
    pub nodes: [MerkleNode<N>; NODES],
    pub leaf_count: usize,
    _phantom: PhantomData<H>,
}

#[macro_export]
macro_rules! merkle_tree {
    ($hash_fn:ty, $depth:expr) => {{
        const N: usize = <$hash_fn>::HASH_BYTES_LEN;
        const NODES: usize = nodes_from_depth($depth);
        MerkleTree::<$hash_fn, N, NODES> {
            root: MerkleNode::default(),
            nodes: [MerkleNode::default(); NODES],
            leaf_count: 0,
            _phantom: core::marker::PhantomData::<$hash_fn>,
        }
    }};
}

impl<H: MerkleHash<N>, const N: usize, const NODES: usize> MerkleTree<H, N, NODES> {
    pub const LEAVES_COUNT: usize = (NODES + 1) / 2;
    pub const LEVELS: u32 = Self::LEAVES_COUNT.ilog2();

    pub fn merklized(&self) -> Result<()> {
        if self.root.inner().eq(&[0u8; 32]) {
            return Err(MerkleError::TreeNotMerklized.into());
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        self.nodes = [MerkleNode::<N>::default(); NODES];
        self.leaf_count = 0;
    }

    /// Add a single leaf
    pub fn add_leaf(&mut self, leaf: &[u8]) -> Result<()> {
        if self.leaf_count >= Self::LEAVES_COUNT {
            return Err(MerkleError::LeafOutOfRange.into());
        }
        self.nodes[self.leaf_count] = H::hash(leaf);
        self.leaf_count += 1;
        Ok(())
    }

    /// Add multiple leaves at once
    pub fn add_leaves(&mut self, leaves: &[&[u8]]) -> Result<()> {
        for leaf in leaves {
            self.add_leaf(leaf)?;
        }
        Ok(())
    }

    /// Add a single leaf
    pub fn add_hash(&mut self, hash: [u8; N]) -> Result<()> {
        if self.leaf_count >= Self::LEAVES_COUNT {
            return Err(MerkleError::LeafOutOfRange.into());
        }
        self.nodes[self.leaf_count] = MerkleNode::from(hash);
        self.leaf_count += 1;
        Ok(())
    }

    /// Add multiple leaves at once
    pub fn add_hashes(&mut self, nodes: &[[u8; N]]) -> Result<()> {
        for hash in nodes {
            self.add_hash(*hash)?;
        }
        Ok(())
    }

    pub fn merklize(&mut self) {
        let mut offset = 0;

        // Go from deepest level (leaves) up to root
        for level in (0..=Self::LEVELS).rev() {
            let current_level_nodes = 1 << level; // 2^level

            let parent_start = offset + current_level_nodes;
            let mut parent_idx = parent_start;

            // process in pairs
            for i in (0..current_level_nodes - 1).step_by(2) {
                let left_idx = offset + i;
                let right_idx = offset + i + 1;

                let parent = {
                    let left = self.nodes[left_idx];
                    let right = self.nodes[right_idx];

                    match (left.is_not_occupied(), right.is_not_occupied()) {
                        (false, false) => H::merkle_hashv(left.inner(), right.inner(), false),
                        (false, true) => H::merkle_hashv(left.inner(), left.inner(), false),
                        (true, false) => H::merkle_hashv(right.inner(), left.inner(), false), // this case wont occur
                        (true, true) => MerkleNode::default(),
                    }
                };

                println!("parent idx: {}", parent_idx);
                self.nodes[parent_idx] = parent;
                parent_idx += 1;
            }

            offset += current_level_nodes; // move offset up for next level
            println!("next offset: {}", offset);
        }

        if let Some(last) = self.nodes.last() {
            self.root = *last;
        }
    }

    /// Get the Merkle root
    pub fn get_merkle_root(&self) -> Result<MerkleNode<N>> {
        if self.root == MerkleNode::<N>::default() {
            return Err(MerkleError::TreeNotMerklized.into());
        }
        Ok(self.root.clone())
    }

    /// Get a leaf hash by index
    pub fn get_leaf_hash(&self, i: usize) -> Result<MerkleNode<N>> {
        if i >= self.leaf_count {
            return Err(MerkleError::LeafOutOfRange.into());
        }
        Ok(self.nodes[i].clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::{nodes_from_depth, MerkleNode, Sha256, Sha256d};
    use hex_literal::hex;

    use super::MerkleTree;

    #[test]
    fn merkle_tree_block_9_test() {
        let mut merkle_tree = MerkleTree::<Sha256, 32, 3> {
            root: MerkleNode::default(),
            nodes: [MerkleNode::default(); 3],
            leaf_count: 0,
            _phantom: core::marker::PhantomData,
        };
        merkle_tree
            .add_hash(hex!(
                "c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704"
            ))
            .unwrap();
        merkle_tree.merklize();

        println!("merkle : {:#?}", merkle_tree);

        assert_eq!(
            &hex!("743f9e7e92165bad517d72503dae64ceba4d831eec8b77e9032cbb70049f1263"),
            merkle_tree.root.inner()
        );
    }

    #[test]
    fn merkle_tree_block_11_test() {
        let mut merkle_tree = MerkleTree::<Sha256, 32, 3> {
            root: MerkleNode::default(),
            nodes: [MerkleNode::default(); 3],
            leaf_count: 0,
            _phantom: core::marker::PhantomData,
        };
        merkle_tree
            .add_hash(hex!(
                "c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704"
            ))
            .unwrap();

        merkle_tree
            .add_hash(hex!(
                "0000000000000000000000000000000000000000000000000000000000000000"
            ))
            .unwrap();

        merkle_tree.merklize();

        println!("merkle : {:#?}", merkle_tree);

        assert_eq!(
            &hex!("02549dd194d947a20a579d0942769759eb726d7a68d15505827260c19ad8260e"),
            merkle_tree.root.inner()
        );
    }

    #[test]
    fn merkle_tree_bitcoin_block_100000_test() {
        const NODES: usize = nodes_from_depth(2);
        let mut merkle_tree = MerkleTree::<Sha256d, 32, NODES> {
            root: MerkleNode::default(),
            nodes: [MerkleNode::default(); NODES],
            leaf_count: 0,
            _phantom: core::marker::PhantomData,
        };

        merkle_tree
            .add_hashes(&[
                hex!("876dd0a3ef4a2816ffd1c12ab649825a958b0ff3bb3d6f3e1250f13ddbf0148c"),
                hex!("c40297f730dd7b5a99567eb8d27b78758f607507c52292d02d4031895b52f2ff"),
                hex!("c46e239ab7d28e2c019b6d66ad8fae98a56ef1f21aeecb94d1b1718186f05963"),
                hex!("1d0cb83721529a062d9675b98d6e5c587e4a770fc84ed00abc5a5de04568a6e9"),
            ])
            .unwrap();

        merkle_tree.merklize();
        println!("merkle : {:#?}", merkle_tree);
        assert_eq!(
            &hex!("6657a9252aacd5c0b2940996ecff952228c3067cc38d4885efb5a4ac4247e9f3"),
            merkle_tree.root.inner()
        );
    }
    #[test]
    fn merkle_tree_bitcoin_block_100002_test() {
        // Build a MerkleTree with depth 4 for example (adjust as needed)
        const DEPTH: usize = 4;
        let mut merkle_tree = merkle_tree!(Sha256d, DEPTH); // use your macro

        // Add leaf hashes as slices
        merkle_tree
            .add_hashes(&[
                hex!("a3f3ac605d5e4727f4ea72e9346a5d586f0231460fd52ad9895bc8240d871def"),
                hex!("076d0317ee70ee36cf396a9871ab3bf6f8e6d538d7f8a9062437dcb71c75fcf9"),
                hex!("2ee1e12587e497ada70d9bd10d31e83f0a924825b96cb8d04e8936d793fb60db"),
                hex!("7ad8b910d0c7ba2369bc7f18bb53d80e1869ba2c32274996cebe1ae264bc0e22"),
                hex!("4e3f8ef2e91349a9059cb4f01e54ab2597c1387161d3da89919f7ea6acdbb371"),
                hex!("e0c28dbf9f266a8997e1a02ef44af3a1ee48202253d86161d71282d01e5e30fe"),
                hex!("8719e60a59869e70a7a7a5d4ff6ceb979cd5abe60721d4402aaf365719ebd221"),
                hex!("5310aedf9c8068f1e862ac9186724f7fdedb0aa9819833af4f4016fca6d21fdd"),
                hex!("201f4587ec86b58297edc2dd32d6fcd998aa794308aac802a8af3be0e081d674"),
            ])
            .unwrap();

        // Compute the Merkle root
        merkle_tree.merklize();

        println!("merkle : {:#?}", merkle_tree);

        // Compare root as bytes
        assert_eq!(
            &hex!("5275289558f51c9966699404ae2294730c3c9f9bda53523ce50e9b95e558da2f"),
            merkle_tree.root.inner()
        );
    }

    #[test]
    fn merkle_tree_payout_test() {
        // Create tree with depth 2 (adjust based on number of leaves)
        const DEPTH: usize = 1;
        let mut merkle_tree = merkle_tree!(Sha256d, DEPTH); // use your macro

        struct Account {
            chain: u16,
            address: Vec<u8>,
            amount: u64,
        }

        impl Account {
            pub fn to_bytes(&self) -> Vec<u8> {
                let mut m = self.chain.to_le_bytes().to_vec();
                m.push(self.address.len() as u8);
                m.extend_from_slice(&self.address);
                m.extend_from_slice(&self.amount.to_le_bytes());
                m
            }
        }

        let leaf_1 = Account {
            chain: 1,
            address: hex!("c0ffee254729296a45a3885639AC7E10F9d54979").to_vec(),
            amount: 1337,
        }
        .to_bytes();

        let leaf_2 = Account {
            chain: 1,
            address: hex!("999999cf1046e68e36E1aA2E0E07105eDDD1f08E").to_vec(),
            amount: 1337,
        }
        .to_bytes();

        merkle_tree.add_leaf(&leaf_1).unwrap();
        merkle_tree.add_leaf(&leaf_2).unwrap();

        // Check leaves
        assert_eq!(
            &hex!("59f9111666f968b79593c142694cb662"),
            &merkle_tree.nodes[0].inner()[..16]
        );
        assert_eq!(
            &hex!("61ebf6f4d1af532451e53c2d2a303390"),
            &merkle_tree.nodes[1].inner()[..16]
        );

        merkle_tree.merklize();

        println!("merkle tree : {:#?}", merkle_tree);
        // Check root
        assert_eq!(
            &hex!(
                "f612a99a5704cf2b7af93ca9c1299453078a6bfed379486aea1c3d39023c6e8e
"
            ),
            merkle_tree.root.inner()
        );
    }

    // #[test]
    // fn merkle_tree_payout_test() {
    //     let mut merkle_tree = MerkleTree::new(crate::HashingAlgorithm::Sha256, 16);

    //     struct Account {
    //         chain: u16,
    //         address: Vec<u8>,
    //         amount: u64,
    //     }

    //     impl Account {
    //         pub fn to_bytes(&self) -> Vec<u8> {
    //             let mut m = self.chain.to_le_bytes().to_vec();
    //             m.extend_from_slice(&[self.address.len() as u8]);
    //             m.extend_from_slice(&self.address);
    //             m.extend_from_slice(&self.amount.to_le_bytes());
    //             m
    //         }
    //     }

    //     let leaf_1 = Account {
    //         chain: 1,
    //         address: hex!("c0ffee254729296a45a3885639AC7E10F9d54979").to_vec(),
    //         amount: 1337,
    //     }
    //     .to_bytes();
    //     let leaf_2 = Account {
    //         chain: 1,
    //         address: hex!("999999cf1046e68e36E1aA2E0E07105eDDD1f08E").to_vec(),
    //         amount: 1337,
    //     }
    //     .to_bytes();

    //     merkle_tree.add_leaf(&leaf_1);
    //     merkle_tree.add_leaf(&leaf_2);

    //     merkle_tree.merklize().unwrap();

    //     assert_eq!(
    //         hex!("59f9111666f968b79593c142694cb662").to_vec(),
    //         merkle_tree.nodes[0][0]
    //     );
    //     assert_eq!(
    //         hex!("61ebf6f4d1af532451e53c2d2a303390").to_vec(),
    //         merkle_tree.nodes[0][1]
    //     );
    //     assert_eq!(
    //         hex!("ed89c53c2635102579a7a002249f7c97460d31ef72baaafd6960be39546c6002").to_vec(),
    //         merkle_tree.root
    //     );

    //     let proof = merkle_tree.merkle_proof_index(0).unwrap();
    //     assert_eq!(merkle_tree.root, proof.merklize(&leaf_1).unwrap());
    //     let proof2 = merkle_tree.merkle_proof_index(1).unwrap();
    //     assert_eq!(merkle_tree.root, proof2.merklize(&leaf_2).unwrap());
    // }

    // #[test]
    // fn test_airdrop() {
    //     let mut merkle_tree = MerkleTree::new(crate::HashingAlgorithm::Sha256, 20);
    //     merkle_tree
    //         .add_leaves(&vec![
    //             hex!("00000000010039050000000000004cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29").to_vec(), // Sol
    //             hex!("01000000020039050000000000007e5f4552091a69125d5dfcb7b8c2659029395bdf").to_vec(), // Eth
    //             hex!("0200000021003905000000000000d0c2c91eda34bbfbaec6cfb9c7bb913e57dab3cbec4018a4b3f5e55531cd63af").to_vec(), // Sui
    //             hex!("03000000220039050000000000004cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29").to_vec() // Aptos
    //         ])
    //         .unwrap();
    //     merkle_tree.merklize().unwrap();

    //     let proof = MerkleProof::new(
    //         HashingAlgorithm::Sha256,
    //         20,
    //         0,
    //         merkle_tree
    //             .merkle_proof_index(0)
    //             .unwrap()
    //             .get_pairing_hashes(),
    //     );
    //     let _proof_root = proof.merklize(&hex!("00000000010039050000000000004cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29")).unwrap();
    //     // format!("{:?}", hex::encode(merkle_tree.get_merkle_root().unwrap()));
    //     // format!("{:?}", hex::encode(proof_root))
    // }
}
