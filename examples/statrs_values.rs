use statrs::statistics::{Data, OrderStatistics};

fn main() {
    // Test data 1: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
    let data1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let mut d1 = Data::new(data1);
    println!("Data [1..10]:");
    println!("  p25: {}", d1.quantile(0.25));
    println!("  p50: {}", d1.quantile(0.50));
    println!("  p75: {}", d1.quantile(0.75));
    println!("  p90: {}", d1.quantile(0.90));
    println!("  p95: {}", d1.quantile(0.95));
    println!("  p99: {}", d1.quantile(0.99));
    
    // Test data 2: [1.0, 2.0, 3.0]
    let data2 = vec![1.0, 2.0, 3.0];
    let mut d2 = Data::new(data2);
    println!("\nData [1.0, 2.0, 3.0]:");
    println!("  p25: {}", d2.quantile(0.25));
    println!("  p50: {}", d2.quantile(0.50));
    println!("  p75: {}", d2.quantile(0.75));
}
