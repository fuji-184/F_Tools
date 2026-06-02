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

ftest::test!(async_graceful_shutdown_tests, {
    test_shutdown_trigger.tokio {
        let has_run = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let has_run_clone = has_run.clone();

        let handle = tokio::spawn(async move {
            async_gracefull_shutdown(move || async move {
                has_run_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            })
            .await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        #[cfg(unix)]
        {
            unsafe {
                libc::kill(libc::getpid(), libc::SIGINT);
            }
        }
        #[cfg(not(unix))]
        {
            return Err("Testing signal handling on Windows is not directly supported via libc::kill".into());
        }

        let _ = handle.await;
        assert!(has_run.load(std::sync::atomic::Ordering::SeqCst));
    }
});