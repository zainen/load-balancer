use std::{thread::sleep, time::Duration};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        // sleep(Duration::from_millis(500))
    }
    sleep(Duration::from_secs(5));

    Ok(())

}
