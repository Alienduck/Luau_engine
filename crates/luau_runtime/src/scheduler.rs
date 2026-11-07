use bevy::prelude::*;
use mlua::Thread;
use std::{cell::RefCell, rc::Rc};

pub struct ScheduledThread {
    pub thread: Thread,
    pub resume_at: f64,
}

#[derive(Clone, Default)]
pub struct SpawnerQueue(pub Rc<RefCell<Vec<(Thread, f64)>>>);

pub struct LuaScheduler {
    pub threads: Vec<ScheduledThread>,
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

pub fn tick_scheduler(world: &mut World) {
    let elapsed = world.resource::<Time>().elapsed_secs_f64();
    let world_ptr = world as *mut World;

    let vm = world.non_send::<crate::vm::LuaVm>();
    vm.lua().set_app_data(world_ptr);

    let mut pending = Vec::new();
    {
        let mut queue = vm.spawner_queue.0.borrow_mut();
        pending.extend(queue.drain(..));
    }

    let mut scheduler = world.non_send_mut::<LuaScheduler>();
    for (thread, delay) in pending {
        scheduler.threads.push(ScheduledThread {
            thread,
            resume_at: elapsed + delay,
        });
    }

    let mut i = 0;
    while i < scheduler.threads.len() {
        if scheduler.threads[i].resume_at > elapsed {
            i += 1;
            continue;
        }
        match scheduler.threads[i].thread.status() {
            mlua::ThreadStatus::Resumable => match scheduler.threads[i]
                .thread
                .resume::<mlua::MultiValue>(())
            {
                Ok(values) => {
                    let mut iter = values.into_iter();
                    match iter.next() {
                        Some(mlua::Value::String(s)) if s.to_str().unwrap() == "wait" => {
                            let secs = iter
                                .next()
                                .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
                                .unwrap_or(0.0);
                            scheduler.threads[i].resume_at = elapsed + secs;
                            i += 1;
                        }
                        _ => {
                            scheduler.threads[i].resume_at = elapsed;
                            i += 1;
                        }
                    }
                }
                Err(e) => {
                    log::error!("[lua] {}", e);
                    scheduler.threads.remove(i);
                }
            },
            mlua::ThreadStatus::Finished | mlua::ThreadStatus::Error => {
                scheduler.threads.remove(i);
            }
            _ => i += 1,
        }
    }

    world
        .non_send::<crate::vm::LuaVm>()
        .lua()
        .remove_app_data::<*mut World>();
}
