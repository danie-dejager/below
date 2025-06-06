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

use std::path::Path;
use std::time::Duration;

use crate::init;

pub struct Statistics {}

impl Statistics {
    pub fn new(_init_token: init::InitToken) -> Statistics {
        Statistics {}
    }

    pub fn report_store_size<P: AsRef<Path>>(&mut self, _dir: P) {}

    pub fn report_nr_accelerators(&mut self, _sample: &model::Sample) {}
}

pub fn report_collection_skew() {}

pub fn report_collection_time_ms(ms: Duration) {}

pub fn report_writer_time_ms(ms: Duration) {}
