mod common;

#[cfg(test)]
mod tests {
    use super::common::init_logger;

    use asiair::ASIAir;
    use asisim::ASIAirSim;
    use std::net::Ipv4Addr;
    use std::time::Duration;
    use rand::Rng;

    #[tokio::test]
    async fn test_camera_config() {
        init_logger();

        // Create a new ASIAir instance
        let addr: Ipv4Addr = Ipv4Addr::from([127, 0, 0, 1]);
        let mut asiair = ASIAir::new(addr);

        // Create a new ASIAir simulator instance
        let mut asiair_sim = ASIAirSim::new();
        asiair_sim.start().await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await; // Give the simulator some time to start

        asiair.connect().await.unwrap();

        let cameras = asiair.get_connected_cameras().await.unwrap();
        println!("Connected cameras: {:?}", cameras);

        assert_eq!(cameras.len(), 2); // Assuming the simulator has two cameras connected
        assert_eq!(cameras[0].name, "ZWO ASI2600MC Pro");
        assert_eq!(cameras[1].name, "ZWO ASI462MM");

        // let camera_state = asiair.get_camera_state().await.unwrap();
        // println!("Camera state: {:?}", camera_state);
        // assert!(matches!(camera_state, CameraState::Close));

        asiair.main_camera_open().await.unwrap();

        // let camera_state = asiair.get_camera_state().await.unwrap();
        // println!("Camera state: {:?}", camera_state);
        // assert!(matches!(camera_state, CameraState::Idle { .. }));

        asiair.main_camera_close().await.unwrap();

        // let camera_state = asiair.get_camera_state().await.unwrap();
        // println!("Camera state: {:?}", camera_state);
        // assert!(matches!(camera_state, CameraState::Close));

        // Open the main camera again
        asiair.main_camera_open().await.unwrap();

        let camera_info = asiair.main_camera_get_info().await.unwrap();
        println!("Camera info: {:?}", camera_info);
        assert_eq!(camera_info.chip_size, [6248, 4176]);
        assert_eq!(camera_info.bins, vec![1, 2, 3, 4]);

        let rand_exposure = rand::rng().random_range(1000..100000000);
        asiair.main_camera_set_exposure(rand_exposure).await.unwrap();

        let exposure = asiair.main_camera_get_exposure().await.unwrap();
        assert_eq!(exposure, rand_exposure);

        let temperature = asiair.main_camera_get_temperature().await.unwrap();
        println!("Camera temperature: {:?}", temperature);

        let cooler : bool = rand::rng().random();
        asiair.main_camera_set_cooler(cooler).await.unwrap();
        let cooler_state = asiair.main_camera_get_cooler().await.unwrap();
        assert_eq!(cooler_state, cooler);

        let gain = rand::rng().random_range(0..100);
        asiair.main_camera_set_gain(gain).await.unwrap();
        let gain_state = asiair.main_camera_get_gain().await.unwrap();
        assert_eq!(gain_state, gain);

        let cooler_percentage_state = asiair.main_camera_get_cooler_percentage().await.unwrap();

        let target_temperature : f64 = rand::rng().random_range(-20..0).into();
        asiair.main_camera_set_target_temperature(target_temperature).await.unwrap();
        let target_temperature_state = asiair.main_camera_get_target_temperature().await.unwrap();
        assert_eq!(target_temperature_state, target_temperature);

        let anti_dew_header : bool = rand::rng().random();
        asiair.main_camera_set_anti_dew_heater(anti_dew_header).await.unwrap();
        let anti_dew_header_state = asiair.main_camera_get_anti_dew_heater().await.unwrap();
        assert_eq!(anti_dew_header_state, anti_dew_header);

        let red_gain = rand::rng().random_range(0..100);
        asiair.main_camera_set_red_gain(red_gain).await.unwrap();
        let red_gain_state = asiair.main_camera_get_red_gain().await.unwrap();
        assert_eq!(red_gain_state, red_gain);

        let blue_gain = rand::rng().random_range(0..100);
        asiair.main_camera_set_blue_gain(blue_gain).await.unwrap();
        let blue_gain_state = asiair.main_camera_get_blue_gain().await.unwrap();
        assert_eq!(blue_gain_state, blue_gain);

        let mono_bin : bool = rand::rng().random();
        asiair.main_camera_set_mono_bin(mono_bin).await.unwrap();
        let mono_bin_state = asiair.main_camera_get_mono_bin().await.unwrap();
        assert_eq!(mono_bin_state, mono_bin);

        let bin = rand::rng().random_range(1..4);
        asiair.main_camera_set_bin(bin).await.unwrap();
        let bin_state = asiair.main_camera_get_bin().await.unwrap();
        assert_eq!(bin_state, bin);

        asiair.main_camera_start_exposure().await.unwrap();
        asiair.main_camera_get_current_img().await.unwrap();

        // Final cleanup
        asiair.disconnect().await;
        asiair_sim.shutdown();
    }
}
