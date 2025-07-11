// Copyright (c) Facebook, Inc. and its affiliates.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;
use std::sync::Mutex;

use async_trait::async_trait;

use super::*;

// For data collection that should be performed on a different thread
#[async_trait]
pub trait AsyncCollectorPlugin {
    type T;

    // Try to collect a sample of type `T`.
    //
    // On success, this should return `Ok(Some(sample))`.
    //
    // On a error handled by the collector, this should return `Ok(None)`. The
    // function itself should consume the error (e.g. log the error)
    //
    // On unhandled errors, this should return `Err(e)`.
    async fn try_collect(&mut self) -> Result<Option<Self::T>>;
}

type SharedVal<T> = Arc<Mutex<Option<T>>>;

// A wrapper around an `AsyncCollectorPlugin` that allows samples to
// be sent to a `Consumer`.
pub struct AsyncCollector<T, Plugin: AsyncCollectorPlugin<T = T>> {
    shared: SharedVal<T>,
    plugin: Plugin,
}

impl<T, Plugin: AsyncCollectorPlugin<T = T>> AsyncCollector<T, Plugin> {
    fn new(shared: SharedVal<T>, plugin: Plugin) -> Self {
        Self { shared, plugin }
    }

    fn update(&self, value: Option<T>) {
        *self.shared.lock().unwrap() = value;
    }

    // Collect sample and update value shared with consumer. Replaces
    // any existing sample that consumer has not consumed yet.
    //
    // Returns true if data was collected and sent. Returns false if
    // there was a recoverable error. Returns an error if there was an
    // unrecoverable error. Errors are never sent to the consumer.
    pub async fn collect_and_update(&mut self) -> Result<bool> {
        let maybe_sample = match self.plugin.try_collect().await {
            Ok(maybe_sample) => maybe_sample,
            Err(e) => {
                self.update(None);
                return Err(e).context("Collector failed to collect");
            }
        };
        let collected = maybe_sample.is_some();
        self.update(maybe_sample);
        Ok(collected)
    }
}

// A consumer for samples collected from a `AsyncCollector`
pub struct Consumer<T> {
    shared: SharedVal<T>,
}

impl<T> Consumer<T> {
    fn new(shared: SharedVal<T>) -> Self {
        Self { shared }
    }

    // Try to get latest sample of data if it exists.
    pub fn take(&self) -> Option<T> {
        self.shared.lock().unwrap().take()
    }
}

// Create a collector consumer pair for a collector plugin
pub fn collector_consumer<T, Plugin: AsyncCollectorPlugin<T = T>>(
    plugin: Plugin,
) -> (AsyncCollector<T, Plugin>, Consumer<T>) {
    let shared = Arc::new(Mutex::new(None));
    (
        AsyncCollector::new(shared.clone(), plugin),
        Consumer::new(shared),
    )
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::sync::Barrier;
    use std::thread;

    use super::*;

    struct TestCollector {
        counter: u64,
    }

    #[async_trait]
    impl AsyncCollectorPlugin for TestCollector {
        type T = u64;

        async fn try_collect(&mut self) -> Result<Option<u64>> {
            self.counter += 1;
            if self.counter == 3 {
                // Recoverable error
                Ok(None)
            } else if self.counter == 4 {
                // Unrecoverable error
                Err(anyhow!("boom"))
            } else {
                Ok(Some(self.counter))
            }
        }
    }

    #[test]
    fn test_collect_and_consume() {
        let (mut collector, consumer) = collector_consumer(TestCollector { counter: 0 });
        let barrier = Arc::new(Barrier::new(2));
        let c = barrier.clone();

        let handle = thread::spawn(move || {
            futures::executor::block_on(collector.collect_and_update()).unwrap();
            // Test overwriting sample
            futures::executor::block_on(collector.collect_and_update()).unwrap();
            c.wait(); // <-- 1
            // Consumer checking overwritten sample
            c.wait(); // <-- 2
            // Test sending None
            futures::executor::block_on(collector.collect_and_update()).unwrap();
            c.wait(); // <-- 3
            // Consumer checking None
            c.wait(); // <-- 4
            // Test sending error. Will fail on both collector and consumer threads.
            let is_error = futures::executor::block_on(collector.collect_and_update()).is_err();
            c.wait(); // <-- 5
            assert!(is_error, "Collector did not return an error");
        });
        // Collector overwriting sample
        barrier.wait(); // <-- 1
        assert_eq!(Some(2), consumer.take());
        barrier.wait(); // <-- 2
        // Collector sending None
        barrier.wait(); // <-- 3
        assert!(consumer.take().is_none());
        barrier.wait(); // <-- 4
        // Collector sending error
        barrier.wait(); // <-- 5
        assert!(consumer.take().is_none());

        handle.join().unwrap();
    }
}
