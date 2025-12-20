// Graph module stub: moved to attic/rust_deadcode/src/graph/
// This stub exists to prevent import errors in dev server (src/main.rs).
// Graph functionality is not part of Tier-0 shipping lanes.

pub mod builder {
    #[derive(Clone, Debug)]
    pub struct GraphNode {
        pub id: String,
    }

    #[derive(Clone, Debug)]
    pub struct GraphEdge {
        pub from: String,
        pub to: String,
    }

    pub fn build_from_episodes(_episodes: &[()]) -> (Vec<GraphNode>, Vec<GraphEdge>) {
        (vec![], vec![])
    }
}

pub mod snapshot {
    pub struct GraphSnapshot;

    impl GraphSnapshot {
        pub fn from_nodes(_nodes: &[()]) -> Self {
            GraphSnapshot
        }
    }
}

// Re-export for backward compat
pub use builder::{GraphNode, GraphEdge};
