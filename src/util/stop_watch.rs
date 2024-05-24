use std::time::Instant;

pub struct StopWatch {
    now: Instant,
}
impl StopWatch {
    pub fn new() -> StopWatch {
        StopWatch {
            now: Instant::now(),
        }
    }
    pub fn log_execution_duration(&self, task: &str) {
        println!("{} - completed:{} ms", task, self.get_ms_duration());
    }
    fn get_ms_duration(&self) -> u128 {
        self.now.elapsed().as_millis()
    }
}
