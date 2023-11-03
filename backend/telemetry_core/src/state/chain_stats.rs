// Source code for the Substrate Telemetry Server.
// Copyright (C) 2022 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use super::counter::{Counter, CounterValue};
use crate::feed_message::ChainStats;

// These are the benchmark scores generated on our reference hardware.
const REFERENCE_CPU_SCORE: u64 = 1028;
const REFERENCE_MEMORY_SCORE: u64 = 14899;
const REFERENCE_DISK_SEQUENTIAL_WRITE_SCORE: u64 = 485;
const REFERENCE_DISK_RANDOM_WRITE_SCORE: u64 = 222;

macro_rules! buckets {
    (@try $value:expr, $bucket_min:expr, $bucket_max:expr,) => {
        if $value < $bucket_max {
            return ($bucket_min, Some($bucket_max));
        }
    };

    ($value:expr, $bucket_min:expr, $bucket_max:expr, $($remaining:expr,)*) => {
        buckets! { @try $value, $bucket_min, $bucket_max, }
        buckets! { $value, $bucket_max, $($remaining,)* }
    };

    ($value:expr, $bucket_last:expr,) => {
        ($bucket_last, None)
    }
}

/// Translates a given raw benchmark score into a relative measure
/// of how the score compares to the reference score.
///
/// The value returned is the range (in percent) within which the given score
/// falls into. For example, a value of `(90, Some(110))` means that the score
/// is between 90% and 110% of the reference score, with the lower bound being
/// inclusive and the upper bound being exclusive.
fn bucket_score(score: u64, reference_score: u64) -> (u32, Option<u32>) {
    let relative_score = ((score as f64 / reference_score as f64) * 100.0) as u32;

    buckets! {
        relative_score,
        0,
        10,
        30,
        50,
        70,
        90,
        110,
        130,
        150,
        200,
        300,
        400,
        500,
    }
}

#[test]
fn test_bucket_score() {
    assert_eq!(bucket_score(0, 100), (0, Some(10)));
    assert_eq!(bucket_score(9, 100), (0, Some(10)));
    assert_eq!(bucket_score(10, 100), (10, Some(30)));
    assert_eq!(bucket_score(29, 100), (10, Some(30)));
    assert_eq!(bucket_score(30, 100), (30, Some(50)));
    assert_eq!(bucket_score(100, 100), (90, Some(110)));
    assert_eq!(bucket_score(500, 100), (500, None));
}

fn bucket_memory(memory: u64) -> (u32, Option<u32>) {
    let memory = memory / (1024 * 1024) / 1000;

    buckets! {
        memory,
        1,
        2,
        4,
        6,
        8,
        10,
        16,
        24,
        32,
        48,
        56,
        64,
        128,
    }
}

fn kernel_version_number(version: &Box<str>) -> &str {
    let index = version
        .find("-")
        .or_else(|| version.find("+"))
        .unwrap_or(version.len());

    &version[0..index]
}

#[test]
fn test_kernel_version_number() {
    assert_eq!(kernel_version_number(&"5.10.0-8-amd64".into()), "5.10.0");
    // Plus sign indicates that the kernel was built from modified sources.
    // This should only appear at the end of the version string.
    assert_eq!(kernel_version_number(&"5.10.0+82453".into()), "5.10.0");
    assert_eq!(kernel_version_number(&"5.10.0".into()), "5.10.0");
}

fn cpu_vendor(cpu: &Box<str>) -> &str {
    let lowercase_cpu = cpu.to_ascii_lowercase();

    if lowercase_cpu.contains("intel") {
        "Intel"
    } else if lowercase_cpu.contains("amd") {
        "AMD"
    } else if lowercase_cpu.contains("arm") {
        "ARM"
    } else if lowercase_cpu.contains("apple") {
        "Apple"
    } else {
        "Other"
    }
}

#[derive(Default)]
pub struct ChainStatsCollator {
    version: Counter<String>,
    target_os: Counter<String>,
    target_arch: Counter<String>,
    cpu: Counter<String>,
    memory: Counter<(u32, Option<u32>)>,
    core_count: Counter<u32>,
    linux_kernel: Counter<String>,
    linux_distro: Counter<String>,
    is_virtual_machine: Counter<bool>,
    cpu_hashrate_score: Counter<(u32, Option<u32>)>,
    memory_memcpy_score: Counter<(u32, Option<u32>)>,
    disk_sequential_write_score: Counter<(u32, Option<u32>)>,
    disk_random_write_score: Counter<(u32, Option<u32>)>,
    cpu_vendor: Counter<String>,
}

impl ChainStatsCollator {
    pub fn add_or_remove_node(
        &mut self,
        details: &common::node_types::NodeDetails,
        hwbench: Option<&common::node_types::NodeHwBench>,
        op: CounterValue,
    ) {
        self.version.modify(Some(&*details.version), op);

        self.target_os
            .modify(details.target_os.as_ref().map(|value| &**value), op);

        self.target_arch
            .modify(details.target_arch.as_ref().map(|value| &**value), op);

        let sysinfo = details.sysinfo.as_ref();
        self.cpu.modify(
            sysinfo
                .and_then(|sysinfo| sysinfo.cpu.as_ref())
                .map(|value| &**value),
            op,
        );

        let memory = sysinfo.and_then(|sysinfo| sysinfo.memory.map(bucket_memory));
        self.memory.modify(memory.as_ref(), op);

        self.core_count
            .modify(sysinfo.and_then(|sysinfo| sysinfo.core_count.as_ref()), op);

        self.linux_kernel.modify(
            sysinfo
                .and_then(|sysinfo| sysinfo.linux_kernel.as_ref())
                .map(kernel_version_number),
            op,
        );

        self.linux_distro.modify(
            sysinfo
                .and_then(|sysinfo| sysinfo.linux_distro.as_ref())
                .map(|value| &**value),
            op,
        );

        self.is_virtual_machine.modify(
            sysinfo.and_then(|sysinfo| sysinfo.is_virtual_machine.as_ref()),
            op,
        );

        self.cpu_vendor.modify(
            sysinfo.and_then(|sysinfo| sysinfo.cpu.as_ref().map(cpu_vendor)),
            op,
        );

        self.update_hwbench(hwbench, op);
    }

    pub fn update_hwbench(
        &mut self,
        hwbench: Option<&common::node_types::NodeHwBench>,
        op: CounterValue,
    ) {
        self.cpu_hashrate_score.modify(
            hwbench
                .map(|hwbench| bucket_score(hwbench.cpu_hashrate_score, REFERENCE_CPU_SCORE))
                .as_ref(),
            op,
        );

        self.memory_memcpy_score.modify(
            hwbench
                .map(|hwbench| bucket_score(hwbench.memory_memcpy_score, REFERENCE_MEMORY_SCORE))
                .as_ref(),
            op,
        );

        self.disk_sequential_write_score.modify(
            hwbench
                .and_then(|hwbench| hwbench.disk_sequential_write_score)
                .map(|score| bucket_score(score, REFERENCE_DISK_SEQUENTIAL_WRITE_SCORE))
                .as_ref(),
            op,
        );

        self.disk_random_write_score.modify(
            hwbench
                .and_then(|hwbench| hwbench.disk_random_write_score)
                .map(|score| bucket_score(score, REFERENCE_DISK_RANDOM_WRITE_SCORE))
                .as_ref(),
            op,
        );
    }

    pub fn generate(&self) -> ChainStats {
        ChainStats {
            version: self.version.generate_ranking_top(10),
            target_os: self.target_os.generate_ranking_top(10),
            target_arch: self.target_arch.generate_ranking_top(10),
            cpu: self.cpu.generate_ranking_top(10),
            memory: self.memory.generate_ranking_ordered(),
            core_count: self.core_count.generate_ranking_top(10),
            linux_kernel: self.linux_kernel.generate_ranking_top(10),
            linux_distro: self.linux_distro.generate_ranking_top(10),
            is_virtual_machine: self.is_virtual_machine.generate_ranking_ordered(),
            cpu_hashrate_score: self.cpu_hashrate_score.generate_ranking_top(10),
            memory_memcpy_score: self.memory_memcpy_score.generate_ranking_ordered(),
            disk_sequential_write_score: self
                .disk_sequential_write_score
                .generate_ranking_ordered(),
            disk_random_write_score: self.disk_random_write_score.generate_ranking_ordered(),
            cpu_vendor: self.cpu_vendor.generate_ranking_top(10),
        }
    }
}
