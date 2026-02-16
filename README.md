# 🧠 Neurographite

**Neuromorphic Spiking Hypergraph Database for Dynamic Network Intelligence**

Neurographite combines hypergraph data structures with neuromorphic computing principles to create a database that learns and adapts relationship strengths over time through spiking neural network patterns.

## 🚀 Features

### Core Architecture
- **Hypergraph Storage**: Nodes connected by hyperedges that link multiple entities simultaneously
- **Spiking Neural Networks**: Temporal dynamics and learning through neural activation patterns  
- **Network Effect Analysis**: Discover cascading effects and goal alignments
- **Stable Matching**: Implement Shapley's algorithm for optimal relationship discovery

### Key Capabilities
- **Dynamic Relationship Learning**: Edge weights adapt based on usage and success patterns
- **Temporal Processing**: Refractory periods and spike propagation for realistic neural modeling
- **Goal Alignment Discovery**: Find complementary entities for network effect maximization
- **Real-time API**: HTTP interface for integration with external systems

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   HTTP API      │───▶│  Core Database   │───▶│   Storage       │
│                 │    │                  │    │   Engine        │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  Hypergraph     │◀───│  Neural          │───▶│  Network        │
│  Structure      │    │  Processing      │    │  Analysis       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## 🔬 Use Cases

### DeepRuin Network Intelligence
- **Goal Alignment**: Match individuals with complementary economic objectives
- **Network Effects**: Identify cascading value creation opportunities  
- **Relationship Discovery**: Find hidden synergies through vector analysis
- **Dynamic Adaptation**: Learn from successful collaborations to improve future matching

### General Applications
- **Social Network Analysis**: Understand influence propagation and community detection
- **Recommendation Systems**: Neural-based collaborative filtering with temporal dynamics
- **Supply Chain Optimization**: Model complex multi-party relationships and dependencies
- **Knowledge Graphs**: Adaptive relationship strengths based on usage and validation

## 🛠️ Installation

### 🐳 Docker (Recommended)
```bash
git clone https://github.com/deepruin/neurographite.git
cd neurographite

# Quick start with Docker Compose
docker-compose up -d

# Or build and run manually
./build-docker.sh
docker run -p 8080:8080 deepruin/neurographite:latest
```

### 📦 From Source
**Prerequisites:** Rust 1.93.1+ (2021 edition), Git

```bash
git clone https://github.com/deepruin/neurographite.git
cd neurographite
cargo build --release
cargo run
# Server starts on http://127.0.0.1:8080
```

### ☁️ Production Deployment
See [DOCKER.md](DOCKER.md) for complete deployment guide including:
- Docker Compose production setup
- Kubernetes manifests
- Docker Swarm configuration  
- Security hardening
- Monitoring and troubleshooting

## 🔌 API Usage

### Add a Node
```bash
curl -X POST http://localhost:8080/nodes \
  -H "Content-Type: application/json" \
  -d '{"data": {"name": "Alice", "goals": ["investment", "AI"]}, "node_type": "person"}'
```

### Connect Nodes
```bash
curl -X POST http://localhost:8080/edges \
  -H "Content-Type: application/json" \
  -d '{
    "node_ids": ["uuid1", "uuid2"], 
    "relationship": "goal_alignment", 
    "strength": 0.8
  }'
```

### Find Similar Nodes
```bash
curl http://localhost:8080/nodes/{uuid}/similar
```

### Simulate Network Effects
```bash
curl http://localhost:8080/nodes/{uuid}/network-effect
```

### Get Database Stats
```bash
curl http://localhost:8080/stats
```

## 🔥 Pulse Query Language

**Pulse** is NeuroGraphite's native query language designed for neuromorphic hypergraph intelligence. Unlike traditional query languages, Pulse treats queries as **neural activations** that propagate through the network.

### Basic Syntax
```pulse
SPIKE influence FROM "Sam Altman"
THROUGH network(depth=3, decay=0.1)
WHERE node_type = "company" 
COLLECT activated_nodes
RETURN name, activation_strength
ORDER BY activation_strength DESC
```

### Core Operations

- **SPIKE** - Initiate neural activation at specific nodes
- **THROUGH** - Define propagation rules (depth, decay, thresholds)  
- **WHERE** - Filter conditions during propagation
- **COLLECT** - Gather results (nodes, paths, timing, cascades)
- **RETURN** - Format final output

### Advanced Features

**Temporal Dynamics:**
```pulse  
SPIKE signal FROM "breaking_news"
THROUGH network(refractory=500ms)
COLLECT propagation_paths, timing_data
RETURN path, activation_time, cascade_depth
```

**Multi-Source Analysis:**
```pulse
SPIKE activation FROM ["OpenAI", "Anthropic", "DeepMind"]  
THROUGH network PARALLEL
COLLECT cascade_effects
RETURN combined_influence, network_effects
```

**Learning Integration:**
```pulse
TRAIN network ON historical_events
WHERE outcome = "successful"
USING gradient_descent(learning_rate=0.01)
RETURN weight_updates
```

For complete documentation, see [PULSE.md](PULSE.md).

## 🧬 Neural Processing

### Spiking Mechanism
- **Activation Threshold**: Nodes spike when activation > 0.7
- **Refractory Period**: 100ms cooldown after spiking
- **Propagation Decay**: 10% signal loss per hop
- **Temporal Dynamics**: Spike history affects similarity calculations

### Learning Rules
- **Hebbian Learning**: "Nodes that spike together, link together"
- **Weight Adaptation**: Successful relationships strengthen over time
- **Decay Functions**: Unused connections gradually weaken

## 📊 Performance

### Benchmarks
```bash
cargo bench
```

### Optimizations
- **Parallel Processing**: Multi-threaded neural computation using Rayon
- **Memory Efficiency**: Sparse data structures for large graphs
- **Persistence**: Efficient binary serialization with Bincode
- **Async I/O**: Non-blocking operations with Tokio

## 🔧 Configuration

```rust
use neurographite::DatabaseConfig;

let config = DatabaseConfig {
    data_dir: "./data".to_string(),
    spike_threshold: 0.7,
    decay_rate: 0.99,
    refractory_period: 100, // milliseconds
    max_cascade_depth: 10,
    sync_interval: 60, // seconds
};

let db = Database::with_config(config).await?;
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Property-based testing
cargo test --features proptest
```

## 📈 Roadmap

### Phase 1: Foundation ✅
- [x] Core hypergraph structure
- [x] Basic spiking neural processing
- [x] HTTP API interface
- [x] Persistent storage

### Phase 2: Intelligence 🚧
- [ ] Advanced stable matching algorithms
- [ ] Machine learning integration for pattern recognition
- [ ] Temporal pattern analysis
- [ ] Distributed processing

### Phase 3: Scale 📋
- [ ] Cluster deployment
- [ ] Real-time streaming updates
- [ ] Advanced visualization
- [ ] Production monitoring

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`  
5. Open a Pull Request

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🦊 Authors

- **Fox** - *Initial work* - [@deepruin](https://github.com/deepruin)
- **Collin** - *Architecture & Vision* - [@deepruin](https://github.com/deepruin)

---

*Built with 🧠 for the future of network intelligence*