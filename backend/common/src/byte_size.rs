// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
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

use anyhow::{anyhow, Error};

#[derive(Copy, Clone, Debug)]
pub struct ByteSize(usize);

impl ByteSize {
    pub fn new(bytes: usize) -> ByteSize {
        ByteSize(bytes)
    }
    /// Return the number of bytes stored within.
    pub fn num_bytes(self) -> usize {
        self.0
    }
}

impl From<ByteSize> for usize {
    fn from(b: ByteSize) -> Self {
        b.0
    }
}

impl std::str::FromStr for ByteSize {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        match s.find(|c| !char::is_ascii_digit(&c)) {
            // No non-numeric chars; assume bytes then
            None => Ok(ByteSize(s.parse().expect("all ascii digits"))),
            // First non-numeric char
            Some(idx) => {
                let n = s[..idx].parse().expect("all ascii digits");
                let suffix = s[idx..].trim();
                let n = match suffix {
                    "B" | "b" => n,
                    "kB" | "K" | "k" => n * 1000,
                    "MB" | "M" | "m" => n * 1000 * 1000,
                    "GB" | "G" | "g" => n * 1000 * 1000 * 1000,
                    "KiB" | "Ki" => n * 1024,
                    "MiB" | "Mi" => n * 1024 * 1024,
                    "GiB" | "Gi" => n * 1024 * 1024 * 1024,
                    _ => {
                        return Err(anyhow!(
                            "\
                        Cannot parse into bytes; suffix is '{}', but expecting one of \
                        B,b, kB,K,k, MB,M,m, GB,G,g, KiB,Ki, MiB,Mi, GiB,Gi",
                            suffix
                        ))
                    }
                };
                Ok(ByteSize(n))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::byte_size::ByteSize;

    #[test]
    fn can_parse_valid_strings() {
        let cases = vec![
            ("100", 100),
            ("100B", 100),
            ("100b", 100),
            ("20kB", 20 * 1000),
            ("20 kB", 20 * 1000),
            ("20K", 20 * 1000),
            (" 20k", 20 * 1000),
            ("1MB", 1000 * 1000),
            ("1M", 1000 * 1000),
            ("1m", 1000 * 1000),
            ("1 m", 1000 * 1000),
            ("1GB", 1000 * 1000 * 1000),
            ("1G", 1000 * 1000 * 1000),
            ("1g", 1000 * 1000 * 1000),
            ("1KiB", 1024),
            ("1Ki", 1024),
            ("1MiB", 1024 * 1024),
            ("1Mi", 1024 * 1024),
            ("1GiB", 1024 * 1024 * 1024),
            ("1Gi", 1024 * 1024 * 1024),
            (" 1 Gi ", 1024 * 1024 * 1024),
        ];

        for (s, expected) in cases {
            let b: ByteSize = s.parse().unwrap();
            assert_eq!(b.num_bytes(), expected);
        }
    }
}
