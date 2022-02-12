
use chrono::prelude::*;
use chrono::Duration;
use plotters::prelude::*;


mod binance;
mod models;
mod statistics;
#[cfg(test)]
mod test_statistics;
mod utils;


#[tokio::main]
async fn main() {
    let client = utils::get_client();
    let result = binance::get_klines(client.clone(), "1d", "BTCUSDT", 500).await;
    
    let kline_data = match result {
        Some(kline_data) => kline_data,
        _ => {
            panic!("Something went wrong.");
        }
    };
    println!("first result: {:?}", kline_data[0]);


    let dir = "plots-output";
    let filepath = format!("{}/sma15.png", &dir);
    let root = BitMapBackend::new(&filepath, (1280, 960)).into_drawing_area();
    root.fill(&WHITE).expect("Error filling background.");

        
    // Convert timestamp to Date<Local>
    let time_data: Vec<(Date<Local>, f64, f64, f64, f64)> = kline_data
        .iter().rev().take(100)
        .map(|x| (timestamp_to_local_date(x.open_time), x.open, x.high, x.low, x.close))
        .collect();
    println!("TIME{:?}", time_data[0]);
    // Get date range
    let (end_date, start_date) = (
        time_data[0].0 + Duration::days(1),
        time_data[time_data.len() - 1].0 - Duration::days(1),
    );

    // Basic chart configuration
    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(60)
        .y_label_area_size(60)
        .caption(
            "Candles + technical indicators",
            ("sans-serif", 50.0).into_font(),
        )
        .build_cartesian_2d(start_date..end_date, 30000f64..80000f64)
        .unwrap();

    chart
        .configure_mesh()
        .light_line_style(&WHITE)
        .draw()
        .unwrap();

    chart
        .draw_series(time_data.iter().map(|x| {
            CandleStick::new(
                x.0,
                x.1,
                x.2,
                x.3,
                x.4,
                RGBColor(98, 209, 61).filled(),
                RGBColor(209, 61, 61).filled(),
                10,
            )
        }))
        .unwrap();

    let price_data: Vec<f64> = kline_data.iter().rev().take(100).map(|x| x.close).collect();
    let result = statistics::simple_moving_average(&price_data, 26);

    let sma_data = match result {
        Some(data) => data,
        _ => panic!("Calculating SMA failed"),
    };

    println!("SMA: {:?}", sma_data[0]);

    let mut line_data: Vec<(Date<Local>, f64)> = Vec::new();
    for i in 0..sma_data.len() {
        line_data.push((time_data[i].0, sma_data[i] as f64));
    }

    println!("SMA_plot: {:?}", line_data[0]);

    chart
        .draw_series(LineSeries::new(line_data, BLUE.stroke_width(2)))
        .unwrap()
        .label("SMA 26")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    

    let result = statistics::exponential_moving_average(&price_data, 26);

    let ema_data = match result {
        Some(data) => data,
        _ => panic!("Calculating EMA failed"),
    };

println!("EMA: {:?}", ema_data[0]);

let mut line2_data: Vec<(Date<Local>, f64)> = Vec::new();
    for i in 0..ema_data.len() {
        line2_data.push((time_data[i].0, ema_data[i] as f64));
    }

    println!("EMA_plot: {:?}", line2_data[0]);

    chart
        .draw_series(LineSeries::new(line2_data, GREEN.stroke_width(2)))
        .unwrap()
        .label("EMA 26")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));

    let result = statistics::moving_average_convergence_divergence(&price_data, 12, 26, 9);

    let macd_data = match result {
        Some(data) => data,
        _ => panic!("Calculating MACD failed"),
        };

    println!("MACD: {:?} {:?}", macd_data.macd[0], macd_data.signal[0]);

    let mut histo_data: Vec<(Date<Local>, f64)> = Vec::new();
    for i in 0..macd_data.macd.len() {
        histo_data.push((time_data[i].0, macd_data.macd[i] as f64));
    }

    println!("MACD_plot: {:?}", histo_data[0]);

    let typical_price_data: Vec<f64> = kline_data
    .iter()
    .rev()
    .take(100)
    .map(|f| (f.high + f.low + f.close) / 3.0)
    .collect();
let result = statistics::bollinger_bands(&typical_price_data, 20, 2.0);

let boll_data = match result {
    Some(data) => data,
        _ => panic!("Calculating BOLL failed"),
    };

    println!("BOLL_UPPER_BOUND: {:?}", boll_data.upper_bound[0]);

    let result = statistics::relative_strength_index(&price_data, 14);

    let rsi_data = match result {
        Some(data) => data,
        _ => panic!("Calculating RSI failed"),
    };

    println!("RSI: {:?}", rsi_data[0]);

        

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperMiddle)
        .label_font(("sans-serif", 30.0).into_font())
        .background_style(WHITE.filled())
        .draw()
        .unwrap();

        root.present().expect(&format!("Unable to write result to file please make sure directory '{}' exists under the current dir", &dir));

    println!("Plot has been saved to {}", &filepath);
    
}

pub fn timestamp_to_local_date(timestamp_milis: i64) -> Date<Local> {
    let naive = NaiveDateTime::from_timestamp(timestamp_milis / 1000, 0);
    Local.from_utc_datetime(&naive).date()
}