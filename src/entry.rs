use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Idea,
    Todo,
    Note,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Idea => write!(f, "idea"),
            Kind::Todo => write!(f, "todo"),
            Kind::Note => write!(f, "note"),
        }
    }
}

impl FromStr for Kind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "idea" => Ok(Kind::Idea),
            "todo" => Ok(Kind::Todo),
            "note" => Ok(Kind::Note),
            _ => Err(format!("unknown kind: {s}")),
        }
    }
}

impl Kind {
    pub const VARIANTS: &'static [&'static str] = &["idea", "todo", "note"];
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::Low => write!(f, "low"),
            Priority::Medium => write!(f, "medium"),
            Priority::High => write!(f, "high"),
            Priority::Critical => write!(f, "critical"),
        }
    }
}

impl FromStr for Priority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "critical" => Ok(Priority::Critical),
            _ => Err(format!("unknown priority: {s}")),
        }
    }
}

impl Priority {
    pub const VARIANTS: &'static [&'static str] = &["low", "medium", "high", "critical"];
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    Done,
    Archived,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Active => write!(f, "active"),
            Status::Done => write!(f, "done"),
            Status::Archived => write!(f, "archived"),
        }
    }
}

impl FromStr for Status {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Status::Active),
            "done" => Ok(Status::Done),
            "archived" => Ok(Status::Archived),
            _ => Err(format!("unknown status: {s}")),
        }
    }
}

impl Status {
    pub const VARIANTS: &'static [&'static str] = &["active", "done", "archived"];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub title: String,
    pub kind: Kind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_priority")]
    pub priority: Priority,
    #[serde(default = "default_status")]
    pub status: Status,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<NaiveDate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

impl Entry {
    pub fn new(title: String, kind: Kind, priority: Priority, status: Status) -> Self {
        let now = Utc::now();
        Self {
            title,
            kind,
            description: None,
            priority,
            status,
            created: now,
            updated: now,
            tags: Vec::new(),
            deadline: None,
            namespace: None,
        }
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn touch(&mut self) {
        self.updated = Utc::now();
    }

    pub fn normalize(&mut self) -> Result<(), String> {
        self.description = self
            .description
            .as_ref()
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty());

        self.tags = normalize_tags(&self.tags);

        self.namespace = self
            .namespace
            .as_ref()
            .map(|n| normalize_namespace(n))
            .transpose()
            ?
            .flatten();
        Ok(())
    }
}

fn default_priority() -> Priority {
    Priority::Medium
}

fn default_status() -> Status {
    Status::Active
}

pub fn normalize_tags(tags: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for tag in tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = trimmed.to_lowercase();
        if seen.insert(key) {
            out.push(trimmed.to_string());
        }
    }
    out
}

pub fn normalize_namespace(input: &str) -> Result<Option<String>, String> {
    let trimmed = input.trim().trim_matches('/');
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.contains("//") {
        return Err("namespace cannot contain empty segments".to_string());
    }
    let segments: Vec<&str> = trimmed.split('/').collect();
    if segments
        .iter()
        .any(|segment| segment.is_empty() || *segment == "." || *segment == "..")
    {
        return Err("namespace contains invalid path segments".to_string());
    }
    Ok(Some(segments.join("/")))
}
