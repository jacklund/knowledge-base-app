use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // An infinite stream of hangup signals.
    let mut sig = signal(SignalKind::hangup())?;

    // Print whenever a HUP signal is received
    loop {
        sig.recv().await;
        break;
    }

    Ok(())
}
