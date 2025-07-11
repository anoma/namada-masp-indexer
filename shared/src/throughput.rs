use std::time::Instant;

pub struct Throughput {
    last_immediate_instant: Instant,
    last_immediate_value: u64,
    calls_to_immediate: u64,
    last_sum: f64,
    average_period: u64,
    last_average: f64,
}

#[derive(Debug)]
pub struct ThroughputMetrics {
    pub immediate: f64,
    pub average: f64,
}

impl Throughput {
    pub fn new(
        average_period: u64,
        last_immediate_value: Option<impl Into<u64>>,
    ) -> Self {
        Self {
            last_immediate_instant: Instant::now(),
            last_immediate_value: last_immediate_value
                .map_or(0, |value| value.into()),
            calls_to_immediate: 0,
            last_sum: 0f64,
            last_average: 0f64,
            average_period,
        }
    }

    pub fn throughput_per_second(
        &mut self,
        new_value: Option<impl Into<u64>>,
    ) -> ThroughputMetrics {
        let new_value = new_value.map_or_else(
            || self.last_immediate_value + 1,
            |value| value.into(),
        );

        let now = Instant::now();
        let elapsed_secs = now
            .duration_since(self.last_immediate_instant)
            .as_secs_f64();
        let difference = (new_value - self.last_immediate_value) as f64;

        self.last_immediate_instant = now;
        self.last_immediate_value = new_value;

        let immediate = difference / elapsed_secs;

        self.last_sum += immediate;
        self.calls_to_immediate += 1;

        if self.calls_to_immediate == self.average_period {
            let last_sum = std::mem::replace(&mut self.last_sum, 0f64);
            self.calls_to_immediate = 0;
            self.last_average = last_sum / self.average_period as f64;
        }

        ThroughputMetrics {
            immediate,
            average: self.last_average,
        }
    }
}
