use asisim::ASIAirSim;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize the ASIAir simulator

    // TODO: read config file how many and the parameters of the ASIAir
    let asiair_sim = ASIAirSim::new();

    // Start the ASIAir simulator, which spawns two threads for UDP and TCP handling
    asiair_sim.start().await.unwrap();

    // Wait for Ctrl+C signal to terminate
    tokio::signal::ctrl_c().await?;
    println!("Shutting down ASIAir simulator...");

    Ok(())
}
