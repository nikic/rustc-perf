//! Each API endpoint has its own module. The modules contain Request and/or
//! Response structs; these contain the specifications for how to interact
//! with the API.
//!
//! The responses are calculated in the server.rs file.

use database::Benchmark;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::result::Result as StdResult;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct StyledBenchmarkName {
    pub name: Benchmark,
    pub profile: crate::db::Profile,
}

impl fmt::Display for StyledBenchmarkName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.name, self.profile)
    }
}

impl Serialize for StyledBenchmarkName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.collect_str(&self)
    }
}

pub type ServerResult<T> = StdResult<T, String>;

pub mod info {
    use database::Date;
    use serde::Serialize;

    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub struct Response {
        /// Sorted list of statistic names known
        pub stats: Vec<String>,

        /// Chronologically last loaded run date.
        pub as_of: Option<Date>,
    }
}

pub mod dashboard {
    use serde::{Deserialize, Serialize};
    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Cases {
        pub clean_averages: Vec<f64>,
        pub base_incr_averages: Vec<f64>,
        pub clean_incr_averages: Vec<f64>,
        pub println_incr_averages: Vec<f64>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Response {
        pub versions: Vec<String>,
        pub check: Cases,
        pub debug: Cases,
        pub opt: Cases,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommitResponse {
    pub commit: Option<String>,
}

pub mod graph {
    use collector::Bound;
    use serde::{Deserialize, Serialize};
    use std::collections::{HashMap, HashSet};

    #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
    pub struct Request {
        pub start: Bound,
        pub end: Bound,
        pub stat: String,
        pub absolute: bool,
    }

    #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
    pub struct GraphData {
        pub commit: u16,
        pub absolute: f32,
        pub percent: f32,
        pub y: f32,
        pub x: u64,
        pub is_interpolated: bool,
    }

    #[derive(Debug, PartialEq, Clone, Serialize)]
    pub struct Response {
        pub benchmarks: HashMap<String, HashMap<String, Vec<(String, Vec<GraphData>)>>>,
        pub max: HashMap<String, f32>,
        pub colors: Vec<String>,
        pub commits: Vec<String>,
    }

    #[derive(Debug, PartialEq, Clone, Serialize)]
    pub struct Series {
        // y-values
        pub points: Vec<f32>,
        // The index of interpolated coordinates
        pub is_interpolated: HashSet<u16>,
    }

    #[derive(Debug, PartialEq, Clone, Serialize)]
    pub struct NewResponse {
        // (UTC timestamp in seconds, sha)
        pub commits: Vec<(i64, String)>,
        pub benchmarks: HashMap<String, HashMap<database::Profile, HashMap<String, Series>>>,
    }
}

pub mod bootstrap {
    use collector::Bound;
    use hashbrown::HashMap;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
    pub struct Request {
        pub start: Bound,
        pub end: Bound,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct Response {
        // (UTC timestamp, sha)
        pub commits: Vec<(i64, String)>,
        // Optional nanoseconds
        pub by_crate: HashMap<String, Vec<Option<u64>>>,
    }
}

pub mod comparison {
    use crate::comparison;
    use collector::Bound;
    use database::Date;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
    pub struct Request {
        pub start: Bound,
        pub end: Bound,
        pub stat: String,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct Response {
        /// The variance data for each benchmark, if any.
        pub variance: Option<HashMap<String, comparison::BenchmarkVariance>>,
        /// The names for the previous artifact before `a`, if any.
        pub prev: Option<String>,

        pub a: ArtifactData,
        pub b: ArtifactData,

        /// The names for the next artifact after `b`, if any.
        pub next: Option<String>,

        /// If `a` and `b` are adjacent artifacts (i.e., `a` is the parent of
        /// `b`).
        pub is_contiguous: bool,
    }

    /// A serializable wrapper for `comparison::ArtifactData`.
    #[derive(Debug, Clone, Serialize)]
    pub struct ArtifactData {
        pub commit: String,
        pub date: Option<Date>,
        pub pr: Option<u32>,
        pub data: HashMap<String, Vec<(String, f64)>>,
        pub bootstrap: HashMap<String, u64>,
    }
}

pub mod status {
    use crate::load::MissingReason;
    use database::ArtifactId;
    use database::Commit;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
    pub struct BenchmarkStatus {
        pub name: String,
        pub success: bool,
        pub error: Option<String>,
    }

    #[derive(Serialize, Debug)]
    pub struct Step {
        pub step: String,
        pub is_done: bool,
        // Seconds
        pub expected_duration: u64,
        // Seconds since start
        pub current_progress: u64,
    }

    #[derive(Serialize, Debug)]
    pub struct CurrentState {
        pub artifact: ArtifactId,
        pub progress: Vec<Step>,
    }

    #[derive(Serialize, Debug)]
    pub struct Response {
        pub last_commit: Option<Commit>,
        pub benchmarks: Vec<BenchmarkStatus>,
        pub missing: Vec<(Commit, MissingReason)>,
        pub current: Option<CurrentState>,
        // None if no recent end, otherwise seconds since epoch
        pub most_recent_end: Option<i64>,
    }
}

pub mod self_profile_raw {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
    pub struct Request {
        pub commit: String,
        pub benchmark: String,
        pub run_name: String,
        pub cid: Option<i32>,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct Response {
        pub cids: Vec<i32>,
        pub cid: i32,
        pub url: String,
        pub is_tarball: bool,
    }
}

pub mod self_profile {
    use database::QueryLabel;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
    pub struct Request {
        pub commit: String,
        pub base_commit: Option<String>,
        pub benchmark: String,
        pub run_name: String,

        pub sort_idx: String,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct Response {
        pub base_profile: Option<SelfProfile>,
        pub profile: SelfProfile,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct SelfProfile {
        pub totals: QueryData,
        pub query_data: Vec<QueryData>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct QueryData {
        pub label: QueryLabel,
        pub self_time: Duration,
        pub percent_total_time: f32,
        pub number_of_cache_misses: u32,
        pub number_of_cache_hits: u32,
        pub invocation_count: u32,
        pub blocked_time: Duration,
        pub incremental_load_time: Duration,
    }
}

pub mod github {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum Association {
        Owner,
        Member,
        Contributor,
        Collaborator,
        FirstTimer,
        FirstTimeContributor,
        None,
    }

    #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
    pub struct User {
        pub id: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Comment {
        pub html_url: String,
        pub author_association: Association,
        pub user: User,
        pub body: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Issue {
        pub number: u32,
        pub comments_url: String,
        pub repository_url: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Request {
        pub issue: Issue,
        pub comment: Comment,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Response;
}

pub mod triage {
    use collector::Bound;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Request {
        pub start: Bound,
        pub end: Option<Bound>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Response(pub String);
}
