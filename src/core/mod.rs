pub mod pattern;
pub mod registry;
pub mod review_engine;
pub mod project_detector;
pub mod custom_rules;

pub use pattern::{AntiPattern, Language, Severity, DetectionMethod, CodeExample};
pub use review_engine::{ReviewEngine, ReviewViolation};
pub use project_detector::ProjectDetector;
pub use custom_rules::{CustomRulesManager, CustomRule};