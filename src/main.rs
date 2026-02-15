mod core;
mod hypergraph;
mod neural;
mod storage;
mod network;
mod api;

use std::error::Error;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🧠 Starting Neurographite - Neuromorphic Hypergraph Database");
    
    // Initialize the database
    let db = neurographite::Database::new("./data").await?;
    
    info!("🚀 Neurographite database initialized");
    
    // Start API server
    let server = api::Server::new(db);
    server.run("127.0.0.1:8080").await?;
    
    Ok(())
}

pub mod neurographite {
    pub use crate::core::Database;
    pub use crate::hypergraph::{HyperNode, HyperEdge, HyperGraph};
    pub use crate::neural::{SpikeProcessor, NeuralState};
    pub use crate::network::{NetworkEffect, GoalAlignment};
}