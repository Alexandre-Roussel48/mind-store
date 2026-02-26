use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
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
    pub name: String,
    pub kind: Kind,
    pub description: String,
    pub priority: Priority,
    pub status: Status,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<NaiveDate>,
}

impl Entry {
    pub fn new(name: String, kind: Kind, description: String, priority: Priority) -> Self {
        let now = Utc::now();
        Self {
            name,
            kind,
            description,
            priority,
            status: Status::Active,
            created: now,
            updated: now,
            tags: Vec::new(),
            deadline: None,
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
}
