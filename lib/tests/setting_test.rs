
mod common;

#[cfg(test)]
mod tests {
    use super::common::init_logger;

    use std::net::SocketAddr;
    use asiair::ASIAir;
    use asisim::ASIAirSim;
    use std::time::Duration;
    use chrono::DateTime;

    #[tokio::test]
    async fn test_asiair_settings() {

        init_logger();

        // Create a new ASIAir instance
        let addr : SocketAddr = SocketAddr::from(([127, 0, 0, 1], 4720));
        let mut asiair = ASIAir::new(addr);

        // Create a new ASIAir simulator instance
        let mut asiair_sim = ASIAirSim::new();
        asiair_sim.start().await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;  // Give the simulator some time to start


        asiair.connect().await.unwrap();

        let datetime = DateTime::parse_from_rfc3339("2023-10-01T12:00:00+00:00").unwrap();
        let datetime = datetime.with_timezone(&chrono_tz::America::Costa_Rica);
        asiair.set_time(datetime).await.unwrap();

        asiair.set_language(asiair::ASIAirLanguage::English).await.unwrap();

        // Final cleanup
        asiair.disconnect().await;
        asiair_sim.shutdown();
    }
    
}