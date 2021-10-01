use async_parser_codealong::{random_packet, raw_packets_stream, Handler};
use futures::{future::select_all, pin_mut, StreamExt};
use rand::thread_rng;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    // console_subscriber::init();

    // let bad_task = async_blocking_task();
    //let good_task = good_task();

    // let _ = select_all([bad_task /* , good_task */]).await;

    let (producer, rx) = spawn_packets_generator();
    let consumer = read_from_channel_task(rx);
    // let _ = select_all([producer, consumer]).await;
    // will never complete
    producer.join();
}

// previously:
async fn do_stream() {
    let mut handler = Handler::new("STREAM".to_string());
    let packet_data = raw_packets_stream();
    pin_mut!(packet_data);
    while let Some(raw_packet) = packet_data.next().await {
        handler.handle_raw_packet(raw_packet);
    }
}

fn async_blocking_task() -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut handler = Handler::new("BLOCK".to_string());
        loop {
            let raw_packet = random_packet(thread_rng());
            handler.handle_raw_packet(raw_packet);
        }
    })
}

fn spawn_packets_generator() -> (
    std::thread::JoinHandle<()>,
    mpsc::UnboundedReceiver<Vec<u8>>,
) {
    let (tx, rx) = mpsc::unbounded_channel();
    (
        std::thread::spawn(move || {
            while !tx.is_closed() {
                println!("PRODUCING");
                if tx.send(random_packet(thread_rng())).is_err() {
                    println!("toctou bites us again, the other end closed down.");
                    break;
                }
            }
        }),
        rx,
    )
}
fn read_from_channel_task(mut rx: mpsc::UnboundedReceiver<Vec<u8>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut handler = Handler::new("BLOCK".to_string());
        loop {
            match rx.recv().await {
                Some(raw_packet) => handler.handle_raw_packet(raw_packet),
                None => {
                    println!("channel closed");
                    break;
                }
            };
        }
    })
}
