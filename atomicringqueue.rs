use atomicring::AtomicRingQueue;
use std::thread;

mod message;

std::include!("settings.in");

fn seq(cap: usize) {
    let q = AtomicRingQueue::with_capacity(cap);

    for i in 0..MESSAGES {
        loop {
            if q.try_push(message::new(i)).is_ok() {
                break;
            } else {
                thread::yield_now();
            }
        }
    }

    for _ in 0..MESSAGES {
        q.pop();
    }
}

fn spsc(cap: usize) {
    let q = AtomicRingQueue::with_capacity(cap);

    crossbeam::scope(|scope| {
        scope.spawn(|_| {
            for i in 0..MESSAGES {
                loop {
                    if q.try_push(message::new(i)).is_ok() {
                        break;
                    } else {
                        thread::yield_now();
                    }
                }
            }
        });

        for _ in 0..MESSAGES {
            q.pop();
        }
    })
    .unwrap();
}

fn mpsc(cap: usize) {
    let q = AtomicRingQueue::with_capacity(cap);

    crossbeam::scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|_| {
                for i in 0..MESSAGES / THREADS {
                    loop {
                        if q.try_push(message::new(i)).is_ok() {
                            break;
                        } else {
                            thread::yield_now();
                        }
                    }
                }
            });
        }

        for _ in 0..MESSAGES {
            q.pop();
        }
    })
    .unwrap();
}

fn mpmc(cap: usize) {
    let q = AtomicRingQueue::with_capacity(cap);

    crossbeam::scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|_| {
                for i in 0..MESSAGES / THREADS {
                    loop {
                        if q.try_push(message::new(i)).is_ok() {
                            break;
                        } else {
                            thread::yield_now();
                        }
                    }
                }
            });
        }

        for _ in 0..THREADS {
            scope.spawn(|_| {
                for _ in 0..MESSAGES / THREADS {
                    q.pop();
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

    println!("atomicringqueue");

    run!("bounded_mpmc", mpmc(MESSAGES));
    run!("bounded_mpsc", mpsc(MESSAGES));
    run!("bounded_seq", seq(MESSAGES));
    run!("bounded_spsc", spsc(MESSAGES));
}
