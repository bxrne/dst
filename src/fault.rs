use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use crate::config::Config;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Pause,
    Kill,
    Deprive(Tier),
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fault::Pause => write!(f, "pause"),
            Fault::Kill => write!(f, "kill"),
            Fault::Deprive(tier) => write!(f, "deprive:{}", tier),
        }
    }
}

impl FromStr for Fault {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("pause") {
            Ok(Fault::Pause)
        } else if s.eq_ignore_ascii_case("kill") {
            Ok(Fault::Kill)
        } else if let Some(tier_str) = s.strip_prefix("deprive:") {
            let tier = Tier::from_str(tier_str)?;
            Ok(Fault::Deprive(tier))
        } else {
            Err(format!("unknown fault type: {}", s))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    Disk,
    Network,
    Memory,
    Cpu,
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tier::Disk => write!(f, "disk"),
            Tier::Network => write!(f, "network"),
            Tier::Memory => write!(f, "memory"),
            Tier::Cpu => write!(f, "cpu"),
        }
    }
}

impl FromStr for Tier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "disk" => Ok(Tier::Disk),
            "network" => Ok(Tier::Network),
            "memory" => Ok(Tier::Memory),
            "cpu" => Ok(Tier::Cpu),
            _ => Err(format!("unknown tier: {}", s)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WeightedFault {
    pub fault: Fault,
    pub weight: f32,
}

impl WeightedFault {
    pub fn new(fault: Fault, weight: f32) -> Self {
        Self { fault, weight }
    }
}

#[derive(Clone, Debug)]
pub struct StepResult {
    pub fault: Fault,
    pub subject_id: String,
    pub round: usize,
    pub total_rounds: usize,
    pub remaining: usize,
    pub more: bool,
}

pub struct FaultTree {
    weighted_faults: Vec<WeightedFault>,
    subjects: Vec<String>,
    rng: StdRng,
    total_steps: usize,
    current_step: usize,
}

impl FaultTree {
    pub fn new(seed: u64, subjects: Vec<String>, config: &Config) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let total_steps = 1 + (rng.next_u32() % 10) as usize;
        let weighted_faults = Self::build_weighted_faults(&config.fault_weights);

        Self {
            weighted_faults,
            subjects,
            rng,
            total_steps,
            current_step: 0,
        }
    }

    pub fn step(&mut self) -> Option<StepResult> {
        if self.current_step >= self.total_steps || self.subjects.is_empty() {
            return None;
        }

        let fault = self.select_weighted_fault();
        let subject_id = self.select_subject()?;

        self.current_step += 1;

        Some(StepResult {
            fault,
            subject_id,
            round: self.current_step,
            total_rounds: self.total_steps,
            remaining: self.total_steps - self.current_step,
            more: self.current_step < self.total_steps,
        })
    }

    fn build_weighted_faults(weights: &HashMap<String, f32>) -> Vec<WeightedFault> {
        let mut weighted_faults = Vec::new();

        for (name, weight) in weights {
            if let Ok(fault) = Fault::from_str(name) {
                weighted_faults.push(WeightedFault::new(fault, *weight));
            }
        }

        weighted_faults
    }

    fn select_weighted_fault(&mut self) -> Fault {
        if self.weighted_faults.is_empty() {
            return Fault::Pause;
        }

        let total: f32 = self.weighted_faults.iter().map(|wf| wf.weight).sum();
        let r: f32 = self.rng.r#gen();
        let mut r = r * total;

        for wf in &self.weighted_faults {
            r -= wf.weight;
            if r <= 0.0 {
                return wf.fault;
            }
        }

        self.weighted_faults.last().unwrap().fault
    }

    fn select_subject(&mut self) -> Option<String> {
        if self.subjects.is_empty() {
            return None;
        }

        let idx = self.rng.gen_range(0..self.subjects.len());
        Some(self.subjects[idx].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fault_display() {
        assert_eq!(Fault::Pause.to_string(), "pause");
        assert_eq!(Fault::Kill.to_string(), "kill");
        assert_eq!(Fault::Deprive(Tier::Disk).to_string(), "deprive:disk");
    }

    #[test]
    fn test_fault_from_str() {
        assert!(matches!(Fault::from_str("pause"), Ok(Fault::Pause)));
        assert!(matches!(Fault::from_str("kill"), Ok(Fault::Kill)));
        assert!(matches!(
            Fault::from_str("deprive:network"),
            Ok(Fault::Deprive(Tier::Network))
        ));
    }

    #[test]
    fn test_tier_from_str() {
        assert!(matches!(Tier::from_str("disk"), Ok(Tier::Disk)));
        assert!(matches!(Tier::from_str("network"), Ok(Tier::Network)));
        assert!(matches!(Tier::from_str("memory"), Ok(Tier::Memory)));
    }
}
