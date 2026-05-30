use bevy::prelude::*;
use mlua::{MultiValue, Thread, ThreadStatus, Value};

struct ScheduledThread {
    thread: Thread,
    resume_at: f64,
}

/// Holds all live Luau coroutines. Non-Send because Thread is not Send.
pub struct LuaScheduler {
    threads: Vec<ScheduledThread>,
}

impl LuaScheduler {
    pub fn new() -> Self {
        Self {
            threads: Vec::new(),
        }
    }

    pub fn spawn(&mut self, thread: Thread) {
        self.threads.push(ScheduledThread {
            thread,
            resume_at: 0.0,
        });
    }
}

impl Default for LuaScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Bevy system — resumes every coroutine whose wait timer has expired.
pub fn tick_scheduler(mut scheduler: NonSendMut<LuaScheduler>, time: Res<Time>) {
    let elapsed = time.elapsed_secs_f64();
    let mut i = 0;

    while i < scheduler.threads.len() {
        if scheduler.threads[i].resume_at > elapsed {
            i += 1;
            continue;
        }

        match scheduler.threads[i].thread.status() {
            ThreadStatus::Resumable => match scheduler.threads[i].thread.resume::<MultiValue>(()) {
                Ok(values) => {
                    let mut iter = values.into_iter();
                    match iter.next() {
                        Some(Value::String(s)) if s.to_str().unwrap() == "wait" => {
                            let secs = iter
                                .next()
                                .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
                                .unwrap_or(0.0);
                            scheduler.threads[i].resume_at = elapsed + secs;
                            i += 1;
                        }
                        _ => i += 1,
                    }
                }
                Err(e) => {
                    log::error!("[lua] {}", e);
                    scheduler.threads.remove(i);
                }
            },
            ThreadStatus::Finished | ThreadStatus::Error => {
                scheduler.threads.remove(i);
            }
            _ => i += 1,
        }
    }
}
