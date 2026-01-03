/// Mock module initialization
pub mod events_jsonl;
pub mod fixture_gen;

pub use events_jsonl::load_events_jsonl;
pub use fixture_gen::{gen_attack_chain_fixture, gen_flood_fixture, gen_smoke_fixture};
