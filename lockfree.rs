use lockfree::channel;

mod message;

std::include!("settings.in");

use std::thread;

fn seq() {
    let (mut tx, mut rx) = channel::spsc::create();

    for i in 0..MESSAGES {
        tx.send(message::new(i)).unwrap();
    }

    for _ in 0..MESSAGES {
        while rx.recv().is_err() {
            thread::yield_now();
        }
    }
}

fn spsc() {
    let (mut tx, mut rx) = channel::spsc::create();

    crossbeam::scope(|scope| {
        scope.spawn(|_| {
            for i in 0..MESSAGES {
                tx.send(message::new(i)).unwrap();
            }
        });

        for _ in 0..MESSAGES {
            while rx.recv().is_err() {
                thread::yield_now();
            }
        }
    })
    .unwrap();
}

fn mpsc() {
    let (tx, mut rx) = channel::mpsc::create();

    crossbeam::scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|_| {
                for i in 0..MESSAGES / THREADS {
                    tx.send(message::new(i)).unwrap();
                }
            });
        }

        for _ in 0..MESSAGES {
            while rx.recv().is_err() {
                thread::yield_now();
            }
        }
    })
    .unwrap();
}

fn mpmc() {
    let (tx, rx) = channel::mpmc::create();

    crossbeam::scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|_| {
                for i in 0..MESSAGES / THREADS {
                    tx.send(message::new(i)).unwrap();
                }
            });
        }

        for _ in 0..THREADS {
            scope.spawn(|_| {
                for _ in 0..MESSAGES / THREADS {
                    while rx.recv().is_err() {
                        thread::yield_now();
                    }
                }
            });
        }
    })
    .unwrap();
}

fn main() {
    macro_rules! run {
        ($name:expr, $f:expr) => {
            let now = ::std::time::Instant::now();
            $f;
            let elapsed = now.elapsed();
            println!("{},{}", $name, elapsed.as_nanos());
        };
    }

    println!("lockfree");

    run!("unbounded_mpmc", mpmc());
    run!("unbounded_mpsc", mpsc());
    run!("unbounded_seq", seq());
    run!("unbounded_spsc", spsc());
}
