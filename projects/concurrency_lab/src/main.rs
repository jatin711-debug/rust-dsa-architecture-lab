use concurrency_lab::network::run_local_tcp_demo;
use concurrency_lab::{count_words_parallel, count_words_sequential, map_bounded};
use std::error::Error;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut args = std::env::args().skip(1);
    match (args.next().as_deref(), args.next(), args.next()) {
        (Some("text"), Some(path), None) => {
            let text = std::fs::read_to_string(path)?;
            let start = Instant::now();
            let sequential = count_words_sequential(&text);
            let sequential_time = start.elapsed();
            let start = Instant::now();
            let parallel = count_words_parallel(&text);
            let parallel_time = start.elapsed();
            assert_eq!(sequential, parallel);
            println!("{} distinct tokens", parallel.len());
            println!("sequential={sequential_time:?} parallel={parallel_time:?}");
            println!("Use repeated release benchmarks before drawing performance conclusions.");
        }
        (Some("async"), None, None) => {
            let results = map_bounded((0..8).collect(), 3, |id| async move {
                let wait = Duration::from_millis(if id % 3 == 0 { 60 } else { 5 });
                tokio::time::timeout(Duration::from_millis(30), async {
                    tokio::time::sleep(wait).await;
                    id * id
                })
                .await
            })
            .await?;
            for (id, result) in results.into_iter().enumerate() {
                println!("job {id}: {result:?}");
            }
        }
        (Some("tcp"), None, None) => {
            let results = run_local_tcp_demo(vec![5, 1, 4, 2, 3], 2).await?;
            println!("local TCP responses: {results:?}");
        }
        _ => {
            return Err("usage: concurrency_lab text <file> | async | tcp".into());
        }
    }
    Ok(())
}
