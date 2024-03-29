use std::thread;
use std::sync;
use std::collections::HashMap;
use std::sync::mpsc;
use std::marker;
use std::sync::{Arc, Mutex};

trait IFunc {
    fn call(self: Box<Self>);
}

impl<T: FnOnce() + marker::Send + 'static> IFunc for T {
    fn call(self: Box<Self>)
        where T: FnOnce() + marker::Send + 'static {
        (*self)();
    }
}

struct CThread {
    sender: mpsc::Sender<Box<dyn IFunc + marker::Send + 'static>>,
    receiver: mpsc::Receiver<Box<dyn IFunc + marker::Send + 'static>>
}

struct CThreadPool {
    threads: HashMap<String, Arc<Mutex<CThread>>>
}

impl CThreadPool {
    pub fn execute<NC, F, T>(&mut self, key: &str, notFoundCb: NC, callback: F)
        where NC: FnOnce() -> T,
            F: IFunc + marker::Send + 'static,
            T: marker::Send + 'static {
        if let Some(t) = self.threads.get_mut(key) {
            let t = match t.lock() {
                Ok(t) => t,
                Err(_) => return,
            };
            t.sender.send(Box::new(callback));
        } else {
            let (s, r) = mpsc::channel();
            let th = Arc::new(Mutex::new(CThread{
                sender: s,
                receiver: r
            }));
            let obj = notFoundCb();
            {
                let t = th.clone();
                thread::spawn(move || {
                    let t = match t.lock() {
                        Ok(t) => t,
                        Err(_) => return,
                    };
                    let recv = match t.receiver.recv() {
                        Ok(recv) => recv,
                        Err(_) => return,
                    };
                    recv.call();
                });
            }
            self.threads.insert(key.to_string(), th.clone());
            if let Ok(t) = th.clone().lock() {
                t.sender.send(Box::new(callback));
            } else {
                println!("send error");
            }
        }
    }
}

impl CThreadPool {
    pub fn new() -> CThreadPool {
        CThreadPool{
            threads: HashMap::new()
        }
    }
}

