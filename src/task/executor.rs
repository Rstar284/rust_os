use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::task::{Waker, Context, Poll};
use crossbeam_queue::ArrayQueue;
use x86_64::instructions::{interrupts, interrupts::enable_and_hlt};

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

pub struct Executor{
    tasks: BTreeMap<TaskId, Task>,
    task_q: Arc<ArrayQueue<TaskId>>,
    waker_c: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_q: Arc::new(ArrayQueue::new(100)),
            waker_c: BTreeMap::new(),
        }
    }
    pub fn spawn(&mut self, task: Task) {
        let taskid = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task already exists");
        }
        self.task_q.push(taskid).expect("the queue is full");
    }
    fn run_ready_tasks(&mut self) {
        let Self {tasks, task_q, waker_c} = self;
        while let Ok(taskid) = task_q.pop() {
            let task = match tasks.get_mut(&taskid) {
                Some(task) => task,
                None => continue,
            };
            let waker = waker_c.entry(taskid).or_insert_with(|| TaskWaker::new(taskid, task_q.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    tasks.remove(&taskid);
                    waker_c.remove(&taskid);
                }
                Poll::Pending => {}
            }
        }
    }
    pub fn run(&mut self) -> ! {
        loop {self.run_ready_tasks(); self.sleep_idle();}
    }
    pub fn sleep_idle(&self) {
        interrupts::disable();
        if self.task_q.is_empty() {enable_and_hlt();} else {interrupts::enable();}
    }
}