# Rust Synthetic Price Generator
This Rust project generates synthetic financial market data (OHLC format) using Geometric Brownian Motion (GBM) and visualizes it as an interactive candlestick chart. The simulation includes intra-period price movements and produces professional-quality visualizations using Plotly.

![analytics](https://github.com/user-attachments/assets/357e434e-8175-47f7-b79e-20efe107aaf8)

## Features

- 📈 **GBM-based price simulation** - Generate realistic price paths with configurable parameters
- 🕰 **Intra-period dynamics** - Simulate detailed price movements between open/close prices
- 📊 **Interactive visualization** - Produces Plotly candlestick charts with zoom/pan capabilities
- 📅 **Temporal features** - Includes proper date handling and time series generation
- 🔧 **Configurable parameters** - Easily adjust market parameters and simulation characteristics

## Usage

### Prerequisites

- Rust 1.70+ installed
- Cargo package manager


### Installation

1. Clone the repository:
```bash
  git clone https://github.com/samizak/Rust-Synthetic-Price-Generation.git
  cd Rust-Synthetic-Price-Generation
```

### Configuration
Modify the simulation parameters in the `main` function:

```rust
let params = Params {
  n_periods: 252 * 10,    // 10 years of daily data
  start_price: 100.0,     // Initial price
  mu: 0.005,              // Daily drift (0.5%)
  sigma: 0.08,            // Daily volatility (8%)
  n_intra_steps: 500,     // Intraday price points per period
};
```

### Running the Simulation
```bash
cargo run
```

After running the program, open `output.html` to see an interactive candlestick chart showing the generated synthetic prices.
