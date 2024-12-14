use std::{thread::sleep, time::{Duration, Instant}};

use rand::Rng;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    
    let start = Instant::now();
    for n in 0..1000 {
        println!("{}", n);
        tokio::task::spawn(async move {
            println!("calling endpoints: {n}");
            let response = reqwest::get("http://127.0.0.1:3000/work").await.unwrap().text().await;
            match response {
                Ok(resp) => println!("response {n}: {:?}", resp),
                Err(_) => println!("Failed to get response")
            };
        });
        let random_number = rng.clone().gen_range(50..=150);
        sleep(Duration::from_millis(random_number))
    }
    
        sleep(Duration::from_secs(1));
    for n in 0..100 {
        println!("{}", n);
        tokio::task::spawn(async move {
            println!("calling endpoints: {n}");
            let response = reqwest::get("http://127.0.0.1:3000/work").await.unwrap().text().await;
            match response {
                Ok(resp) => println!("response {n}: {:?}", resp),
                Err(_) => println!("Failed to get response")
            };
        });
    }
    println!("-----------------------> total time: {:.2?}", start.elapsed());
    sleep(Duration::from_secs(60));

    Ok(())

}
