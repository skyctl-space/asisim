mod asiair;

use asiair::ASIAirSim;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize the ASIAir simulator

    // TODO: read config file how many and the parameters of the ASIAir
    let asiair_sim = ASIAirSim::new();

    let simtask = tokio::spawn(async move {
        asiair_sim.start().await.unwrap();
    });

    // Wait for both simulators (Ctrl+C will interrupt)
    tokio::try_join!(simtask)?;

    Ok(())
}
