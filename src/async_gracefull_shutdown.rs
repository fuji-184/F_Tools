use std::future::Future;

pub async fn async_gracefull_shutdown<F, Fut>(cleanup_logic: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    let ctrl_c = tokio::signal::ctrl_c();
    
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Shutdown signal received. Running cleanup logic...");
    
    cleanup_logic().await;
    
    println!("Cleanup complete. Exiting");
}