use chrono::{Duration, NaiveDate};
use plotly::{Candlestick, Plot};
use polars::prelude::*;
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
    prices
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

fn generate_new_ohlc(params: &Params) -> DataFrame {
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

    // Convert dates to days since epoch (1970-01-01)
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let dates_as_days: Vec<i32> = date_range
        .iter()
        .map(|d| d.signed_duration_since(epoch).num_days() as i32)
        .collect();

    let mut opens = Vec::with_capacity(params.n_periods);
    let mut highs = Vec::with_capacity(params.n_periods);
    let mut lows = Vec::with_capacity(params.n_periods);
    let mut closes = Vec::with_capacity(params.n_periods);

    let mut prev_close = params.start_price;
    for (_, close) in date_range.iter().zip(close_prices.iter()) {
        let open = prev_close;
        let intra_prices = generate_intra_prices(open, *close, params.n_intra_steps, params.sigma);
        let high = intra_prices.iter().cloned().fold(f64::MIN, f64::max);
        let low = intra_prices.iter().cloned().fold(f64::MAX, f64::min);

        opens.push(open);
        highs.push(high);
        lows.push(low);
        closes.push(*close);
        prev_close = *close;
    }

    DataFrame::new(vec![
        Series::new("date".into(), dates_as_days)
            .cast(&DataType::Date)
            .unwrap()
            .into(), // Convert to Column
        Series::new("open".into(), opens).into(),
        Series::new("high".into(), highs).into(),
        Series::new("low".into(), lows).into(),
        Series::new("close".into(), closes).into(),
    ])
    .unwrap()
}

fn redraw_plot(current_df: &DataFrame) {
    let mut plot = Plot::new();

    // Convert dates back from days since epoch
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let date_series = current_df.column("date").unwrap().date().unwrap();
    let dates: Vec<String> = date_series
        .into_iter()
        .map(|d| {
            epoch
                .checked_add_signed(Duration::days(d.unwrap() as i64))
                .unwrap()
                .format("%Y-%m-%d")
                .to_string()
        })
        .collect();

    let opens: Vec<f64> = current_df
        .column("open")
        .unwrap()
        .f64()
        .unwrap()
        .into_no_null_iter()
        .collect();
    let highs: Vec<f64> = current_df
        .column("high")
        .unwrap()
        .f64()
        .unwrap()
        .into_no_null_iter()
        .collect();
    let lows: Vec<f64> = current_df
        .column("low")
        .unwrap()
        .f64()
        .unwrap()
        .into_no_null_iter()
        .collect();
    let closes: Vec<f64> = current_df
        .column("close")
        .unwrap()
        .f64()
        .unwrap()
        .into_no_null_iter()
        .collect();

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
