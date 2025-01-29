use rand::rng;
use rand_distr::{Distribution, Normal};
use std::f64;

fn generate_gbm(start_price: f64, mu: f64, sigma: f64, n_periods: usize) -> Vec<f64> {
    let mut prices = vec![0.0; n_periods];
    prices[0] = start_price;
    let dt = 1.0;
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut rng = rng();

    for t in 1..n_periods {
        let drift = (mu - 0.5 * sigma.powi(2)) * dt;
        let shock = sigma * (dt.sqrt()) * normal.sample(&mut rng);
        prices[t] = prices[t - 1] * (drift + shock).exp();
    }

    prices
}

fn generate_intra_prices(
    open_price: f64,
    close_price: f64,
    n_steps: usize,
    sigma: f64,
) -> Vec<f64> {
    if n_steps < 2 {
        return vec![open_price, close_price];
    }
    let mut returns = vec![0.0; n_steps - 1];
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut rng = rng();

    for r in &mut returns {
        *r = normal.sample(&mut rng) * (sigma / (n_steps as f64).sqrt());
    }

    let mut prices = vec![open_price];
    let mut current_price = open_price;
    for r in &returns {
        current_price *= (r).exp();
        prices.push(current_price);
    }

    let required_log_return = (close_price / prices[prices.len() - 1]).ln();
    let adjustment = required_log_return / (n_steps - 1) as f64;
    let adjusted_returns: Vec<f64> = returns.iter().map(|&r| r + adjustment).collect();

    let mut prices = vec![open_price];
    current_price = open_price;
    for r in &adjusted_returns {
        current_price *= (r).exp();
        prices.push(current_price);
    }

    prices
}

fn main() {
    let start_price = 100.0;
    let mu = 0.01;
    let sigma = 0.1;
    let n_periods = 252;
    let n_intra_steps = 100;

    let close_prices = generate_gbm(start_price, mu, sigma, n_periods);

    let mut df = vec![(0.0, 0.0, 0.0, 0.0); n_periods];
    df[0].0 = start_price; // Open

    for i in 0..n_periods {
        let open_p = if i == 0 { start_price } else { df[i - 1].3 }; // Previous Close
        let close_p = close_prices[i];
        let intra_prices = generate_intra_prices(open_p, close_p, n_intra_steps, sigma);
        let high = intra_prices
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let low = intra_prices.iter().cloned().fold(f64::INFINITY, f64::min);

        df[i] = (open_p, high, low, close_p);
    }

    // Print the first few rows of the dataframe for verification
    for i in 0..10 {
        println!("{:?}", df[i]);
    }
}
