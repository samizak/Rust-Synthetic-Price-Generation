use chrono::{Duration, NaiveDate};
use plotly::{Candlestick, Plot};
use rand::prelude::*;
use rand_distr::{Normal, StandardNormal};
use std::fs;

// --------------------------
// Data Generation Functions
// --------------------------
fn generate_gbm(start_price: f64, mu: f64, sigma: f64, n_periods: usize) -> Vec<f64> {
    let mut rng = rand::rng();
    let dt = 1.0;
    let mut prices = vec![0.0; n_periods];
    prices[0] = start_price;

    for t in 1..n_periods {
        let drift = (mu - 0.5 * sigma.powi(2)) * dt;
        let shock = sigma * dt.sqrt() * rng.sample::<f64, _>(StandardNormal);
        prices[t] = prices[t - 1] * (drift + shock).exp();
    }
    return prices;
}

fn get_normal_distribution(sigma: f64, n_steps: usize) -> Vec<f64> {
    let mut rng = rand::rng();
    let normal = Normal::new(0.0, sigma / (n_steps as f64).sqrt()).unwrap();
    (0..n_steps - 1).map(|_| normal.sample(&mut rng)).collect()
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

    let returns = get_normal_distribution(sigma, n_steps);
    let mut prices = vec![open_price];
    let mut current_price = open_price;

    for r in &returns {
        current_price *= r.exp();
        prices.push(current_price);
    }

    let required_log_return = (close_price / prices.last().unwrap()).ln();
    let adjustment = required_log_return / ((n_steps - 1) as f64);

    let adjusted_returns: Vec<f64> = returns.iter().map(|&r| r + adjustment).collect();
    prices = vec![open_price];
    current_price = open_price;

    for r in adjusted_returns {
        current_price *= r.exp();
        prices.push(current_price);
    }
    prices
}

// ---------------------
// Main Visualization
// ---------------------
fn main() {
    let params = Params {
        n_periods: 252 * 5,
        start_price: 100.0,
        mu: 0.005,
        sigma: 0.08,
        n_intra_steps: 500,
    };

    let current_df = generate_new_ohlc(&params);
    redraw_plot(&current_df);
}

fn generate_new_ohlc(params: &Params) -> Vec<(NaiveDate, f64, f64, f64, f64)> {
    let close_prices = generate_gbm(
        params.start_price,
        params.mu,
        params.sigma,
        params.n_periods,
    );

    let date_range: Vec<NaiveDate> = (0..params.n_periods)
        .map(|i| {
            let date = NaiveDate::from_ymd_opt(2024, 12, 31).expect("Invalid initial date");
            date.checked_sub_signed(Duration::days(i as i64))
                .expect("Date out of range")
        })
        .rev()
        .collect();

    let mut df = Vec::new();
    let mut prev_close = params.start_price;

    for (date, close) in date_range.iter().zip(close_prices.iter()) {
        let open = prev_close;
        let intra_prices = generate_intra_prices(open, *close, params.n_intra_steps, params.sigma);
        let high = intra_prices.iter().cloned().fold(f64::MIN, f64::max);
        let low = intra_prices.iter().cloned().fold(f64::MAX, f64::min);

        df.push((*date, open, high, low, *close));
        prev_close = *close;
    }
    df
}

fn redraw_plot(current_df: &[(NaiveDate, f64, f64, f64, f64)]) {
    let mut plot = Plot::new();

    // Extract dates
    let dates: Vec<String> = current_df
        .iter()
        .map(|(date, _, _, _, _)| date.format("%Y-%m-%d").to_string())
        .collect();

    // Extract OHLC values
    let opens: Vec<f64> = current_df.iter().map(|(_, o, _, _, _)| *o).collect();
    let highs: Vec<f64> = current_df.iter().map(|(_, _, h, _, _)| *h).collect();
    let lows: Vec<f64> = current_df.iter().map(|(_, _, _, l, _)| *l).collect();
    let closes: Vec<f64> = current_df.iter().map(|(_, _, _, _, c)| *c).collect();

    let trace = Candlestick::new(dates, opens, highs, lows, closes).name("OHLC");

    plot.add_trace(Box::new(trace));

    let html_snippet = plot.to_inline_html(Some("my-plot-id"));
    let template = fs::read_to_string("template.html").expect("Failed to read template.html");
    let final_html = template.replace("<!-- PLOT_PLACEHOLDER -->", &html_snippet);
    fs::write("output.html", final_html).expect("Failed to write output.html");
}

struct Params {
    n_periods: usize,
    start_price: f64,
    mu: f64,
    sigma: f64,
    n_intra_steps: usize,
}
