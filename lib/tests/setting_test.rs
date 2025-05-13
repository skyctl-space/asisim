mod common;

#[cfg(test)]
mod tests {
    use super::common::init_logger;

    use asiair::ASIAir;
    use asisim::ASIAirSim;
    use chrono::DateTime;
    use std::net::Ipv4Addr;
    use std::time::Duration;

    #[tokio::test]
    async fn test_asiair_settings() {
        init_logger();

        // Create a new ASIAir instance
        let addr: Ipv4Addr = Ipv4Addr::from([127, 0, 0, 1]);
        let mut asiair = ASIAir::new(addr);

        // Create a new ASIAir simulator instance
        let mut asiair_sim = ASIAirSim::new();
        asiair_sim.start().await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await; // Give the simulator some time to start

        asiair.connect().await.unwrap();

        let datetime = DateTime::parse_from_rfc3339("2023-10-01T12:00:00+00:00").unwrap();
        let datetime = datetime.with_timezone(&chrono_tz::America::Costa_Rica);
        asiair.set_time(datetime).await.unwrap();

        asiair
            .set_language(asiair::ASIAirLanguage::English)
            .await
            .unwrap();

        // Final cleanup
        asiair.disconnect().await;
        asiair_sim.shutdown();
    }
}
