pub mod custom_rules;
pub mod pattern;
pub mod project_detector;
pub mod registry;
pub mod review_engine;

pub use custom_rules::{CustomRule, CustomRulesManager};
pub use pattern::{AntiPattern, CodeExample, DetectionMethod, Language, Severity};
pub use project_detector::ProjectDetector;
pub use review_engine::{ReviewEngine, ReviewViolation};
