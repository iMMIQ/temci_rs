use statrs::statistics::{Data, OrderStatistics};

fn main() {
    // Test from test_outlier_detection_multiple
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0, 200.0];
    let mut d = Data::new(data);
    let q1 = d.quantile(0.25);
    let q3 = d.quantile(0.75);
    let iqr = q3 - q1;
    let lower = q1 - 1.5 * iqr;
    let upper = q3 + 1.5 * iqr;
    println!("Q1: {}, Q3: {}, IQR: {}", q1, q3, iqr);
    println!("Lower: {}, Upper: {}", lower, upper);
    
    // Check each value
    for val in [1.0, 2.0, 3.0, 4.0, 5.0, 100.0, 200.0] {
        println!("{} is outlier? {}", val, val < lower || val > upper);
    }
}
