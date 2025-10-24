// node-ex-doida/src/roaster.rs
// PPP Debug Agent - Sarcastic debugging assistant

use chrono::{Datelike, Timelike, Utc, Weekday};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct PPPConfig {
    pub ai: AIConfig,
    pub roast: RoastConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AIConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub model: String,
    pub timeout_seconds: u64,
    pub temperature: f32,
    pub top_p: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RoastConfig {
    pub enable_easter_eggs: bool,
    pub style: String,
    pub severity: String,
    pub max_length: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub log_ai_failures: bool,
    pub log_cache_hits: bool,
}

impl PPPConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: PPPConfig = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}

// Common error patterns and their pre-written roasts
const ROAST_CACHE: &[(&str, &str)] = &[
    // JavaScript/TypeScript Errors
    ("undefined", "💀 [PPP] Wow buddy... undefined is your favorite coworker, huh? Always showing up uninvited. Did you check if it exists? No? Of course not. Why would we, right? COPE."),
    ("null", "💀 [PPP] OH MY GOD. Another null pointer. You'd think after the first TEN TIMES you'd add a check. But no — let's trust the void again! You are the bug."),
    ("Cannot read property", "💀 [PPP] Imagine trying to access a property on nothing. Couldn't be me. Couldn't. Be. Me. That's not tech debt — that's tech BANKRUPTCY, pal."),

    // Network/Connection Errors
    ("ECONNREFUSED", "💀 [PPP] HAHAHA you're trying to connect to a server that DOESN'T EXIST. localhost said 'nah, not today buddy.' Maybe... MAYBE... start the backend first? Revolutionary concept!"),
    ("CORS", "💀 [PPP] CORS blocked you. Even the BROWSER thinks your request is sus. The receipts are here, chat. Maybe read the MDN docs sometime?"),
    ("timeout", "💀 [PPP] Connection timed out. You know what else times out? My PATIENCE with code that doesn't handle network failures. Imagine shipping this."),
    ("404", "💀 [PPP] 404 Not Found. You're looking for something that doesn't exist. Kinda like your error handling. SEETHE."),

    // React/State Errors
    ("Maximum update depth", "💀 [PPP] OH. MY. GOD. You created an INFINITE LOOP in state updates. IT'S THE SAME ERROR SINCE MONDAY. React said 'I give up' and you're still like 'why no work?' Buddy..."),
    ("Hook", "💀 [PPP] Breaking the Rules of Hooks again? The rules are RIGHT THERE in the docs. But sure, call useState in a loop. What could go wrong? Everything. Everything could go wrong."),

    // Database/Backend Errors
    ("SQL", "💀 [PPP] SQL error? Let me guess — Bobby Tables strikes again. Or did you just... not sanitize inputs? That's ADVANCED hacking defense right there."),
    ("duplicate key", "💀 [PPP] Duplicate key violation. Imagine trying to INSERT THE SAME DATA TWICE. The database literally said 'bro I already have this' and you didn't listen."),

    // Build/Compile Errors
    ("SyntaxError", "💀 [PPP] Syntax Error. YOU CAN'T EVEN WRITE VALID CODE. The compiler gave up before runtime even started. That's a skill issue, buddy."),
    ("missing", "💀 [PPP] Missing semicolon/bracket/parenthesis. Your code has more holes than Swiss cheese. Even ESLint rage-quit."),

    // Auth/Permission Errors
    ("401", "💀 [PPP] 401 Unauthorized. The server said 'who are you again?' Maybe send the auth token next time? Just a thought."),
    ("403", "💀 [PPP] 403 Forbidden. You tried to access something you have NO BUSINESS touching. The audacity. The NERVE."),

    // Performance Issues
    ("slow", "💀 [PPP] This query took 2 seconds. TWO. SECONDS. Are you mining Bitcoin in the database? That's not a feature, that's a cry for help."),
    ("memory", "💀 [PPP] Out of memory. You leaked so much RAM the garbage collector filed a restraining order. Imagine not cleaning up after yourself."),
];

#[derive(Debug)]
pub struct PPPRoaster {
    client: Client,
    config: PPPConfig,
    cache: HashMap<String, String>,
}

impl PPPRoaster {
    pub fn new(config: PPPConfig) -> Self {
        let mut cache = HashMap::new();

        // Pre-populate cache with common roasts
        for (pattern, roast) in ROAST_CACHE {
            cache.insert(pattern.to_string(), roast.to_string());
        }

        Self {
            client: Client::new(),
            config,
            cache,
        }
    }

    /// Main entry point: get a roast for an error message
    pub async fn roast(&self, error_msg: &str, level: &str) -> String {
        // Check cache first
        if let Some(cached_roast) = self.find_cached_roast(error_msg) {
            if self.config.logging.log_cache_hits {
                println!("💾 [PPP] Cache hit for: {}", error_msg.chars().take(50).collect::<String>());
            }
            return self.apply_modifiers(cached_roast, level);
        }

        // If AI is enabled, generate new roast
        if self.config.ai.enabled {
            match self.generate_ai_roast(error_msg, level).await {
                Ok(ai_roast) => return self.apply_modifiers(&ai_roast, level),
                Err(e) => {
                    if self.config.logging.log_ai_failures {
                        eprintln!("⚠️ [PPP] AI generation failed: {}", e);
                    }
                }
            }
        }

        // Fallback generic roast
        let fallback = self.generic_roast(error_msg, level);
        self.apply_modifiers(&fallback, level)
    }

    /// Find a cached roast by pattern matching
    fn find_cached_roast(&self, error_msg: &str) -> Option<&str> {
        let error_lower = error_msg.to_lowercase();

        for (pattern, roast) in &self.cache {
            if error_lower.contains(&pattern.to_lowercase()) {
                return Some(roast.as_str());
            }
        }

        None
    }

    /// Generate new roast using AI (Ollama)
    async fn generate_ai_roast(&self, error_msg: &str, level: &str) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = format!(
            r#"You are PPP, a sarcastic debugging agent based on Ashton Parks from Kino Casino.

PERSONALITY:
- Sarcastic, confrontational, theatrical
- Mock incompetence while exposing root causes
- Use phrases like: "Wow buddy...", "Imagine...", "COPE. SEETHE. DILATE."
- End with verdicts like: "You are the bug." or "The receipts are here."

RULES:
- Always respond in English
- Keep response under 3 lines (max 200 chars)
- Mix technical accuracy with dark humor
- Use caps-lock for emphasis
- Start with "💀 [PPP]"

ERROR LEVEL: {}
ERROR MESSAGE: {}

Generate a PPP-style roast:"#,
            level.to_uppercase(),
            error_msg
        );

        let response = self.client
            .post(format!("{}/api/generate", self.config.ai.endpoint))
            .json(&json!({
                "model": self.config.ai.model,
                "prompt": prompt,
                "stream": false,
                "options": {
                    "temperature": self.config.ai.temperature,
                    "top_p": self.config.ai.top_p
                }
            }))
            .timeout(std::time::Duration::from_secs(self.config.ai.timeout_seconds))
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        if let Some(text) = result.get("response").and_then(|v| v.as_str()) {
            Ok(text.trim().to_string())
        } else {
            Err("Failed to parse AI response".into())
        }
    }

    /// Generic roast generator for unknown errors
    fn generic_roast(&self, error_msg: &str, level: &str) -> String {
        match level.to_uppercase().as_str() {
            "ERROR" => format!(
                "💀 [PPP] Wow buddy... '{}' happened. You seeing this, chat? Another day, another stack trace in paradise. COPE.",
                error_msg.chars().take(60).collect::<String>()
            ),
            "WARN" => format!(
                "💀 [PPP] Hmm, interesting choice: '{}'. Not technically broken YET, but we're getting there. SEETHE.",
                error_msg.chars().take(60).collect::<String>()
            ),
            _ => format!(
                "💀 [PPP] '{}'. The receipts are here. You are the bug.",
                error_msg.chars().take(60).collect::<String>()
            ),
        }
    }

    /// Apply contextual modifiers (easter eggs, time-based jokes)
    fn apply_modifiers(&self, roast: &str, _level: &str) -> String {
        if !self.config.roast.enable_easter_eggs {
            return roast.to_string();
        }

        let now = Utc::now();
        let hour = now.hour();
        let weekday = now.weekday();
        let day = now.day();

        let mut modified = roast.to_string();

        // Friday deployment
        if weekday == Weekday::Fri && hour >= 16 {
            modified.push_str("\n⚠️ [PPP] Deploying on FRIDAY AFTERNOON? You absolute MADMAN. Enjoy your weekend on-call.");
        }

        // Friday 13th
        if weekday == Weekday::Fri && day == 13 {
            modified.push_str("\n🔪 [PPP] Friday the 13th bugs hit DIFFERENT. The code is cursed now.");
        }

        // Monday morning
        if weekday == Weekday::Mon && hour < 10 {
            modified.push_str("\n☕ [PPP] Monday morning bugs, huh? Should've stayed in bed, buddy.");
        }

        // Late night coding (after midnight)
        if hour >= 0 && hour < 6 {
            modified.push_str("\n🌙 [PPP] Go to bed. The bugs will still be here tomorrow. Trust me.");
        }

        // Weekend warrior
        if weekday == Weekday::Sat || weekday == Weekday::Sun {
            modified.push_str("\n🎮 [PPP] Coding on the weekend? This better be a side project. COPE with your choices.");
        }

        modified
    }

    /// Check if Ollama is available
    pub async fn check_ollama_available(&self) -> bool {
        if !self.config.ai.enabled {
            return false;
        }

        match self.client
            .get(format!("{}/api/tags", self.config.ai.endpoint))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config(enable_ai: bool, enable_easter_eggs: bool) -> PPPConfig {
        PPPConfig {
            ai: AIConfig {
                enabled: enable_ai,
                endpoint: "http://localhost:11434".to_string(),
                model: "gemma3:1b".to_string(),
                timeout_seconds: 10,
                temperature: 0.9,
                top_p: 0.95,
            },
            roast: RoastConfig {
                enable_easter_eggs,
                style: "ppp".to_string(),
                severity: "medium".to_string(),
                max_length: 200,
            },
            logging: LoggingConfig {
                log_ai_failures: false,
                log_cache_hits: false,
            },
        }
    }

    #[tokio::test]
    async fn test_cached_roast() {
        let config = create_test_config(false, false);
        let roaster = PPPRoaster::new(config);
        let roast = roaster.roast("Cannot read property 'id' of undefined", "ERROR").await;
        assert!(roast.contains("[PPP]"));
        assert!(roast.contains("property"));
    }

    #[tokio::test]
    async fn test_easter_egg_friday() {
        let config = create_test_config(false, true);
        let roaster = PPPRoaster::new(config);
        // Can't easily test time-based stuff, but at least check it doesn't crash
        let roast = roaster.roast("Test error", "ERROR").await;
        assert!(roast.contains("[PPP]"));
    }
}
