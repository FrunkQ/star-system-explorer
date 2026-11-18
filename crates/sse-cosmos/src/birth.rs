//! What a newborn system inherits from its patch (galaxy design 8.4;
//! formation design 3, stage 1), as a typed read of its resolved facts.
//! The physics suite's formation stage F0 starts here.

use crate::patch::{GYR_S, MASS_QUANTUM_KG, MSUN_KG};
use serde::{Deserialize, Serialize};
use sse_core::cbor::Value;
use sse_core::Address;
use std::collections::BTreeMap;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SystemBirth {
    pub address: Address,
    pub population: String,
    /// The cloud core's mass, all of which the system's stars share.
    pub mass_msun: f64,
    pub birth_gyr: f64,
    pub feh: f64,
    pub alpha_fe: f64,
    pub c_o: f64,
    pub mg_si: f64,
    /// Relative to the Sun's at its birth.
    pub r_process_rel: f64,
    pub k40_rel: f64,
    pub al26_rel: f64,
    pub cluster_mass_msun: f64,
    pub cluster_fuv_g0: f64,
}

fn num(f: &BTreeMap<String, Value>, k: &str) -> Result<f64, String> {
    match f.get(k) {
        Some(Value::Float(x)) => Ok(*x),
        Some(Value::Int(i)) => Ok(*i as f64),
        _ => Err(format!("missing number {k}")),
    }
}

impl SystemBirth {
    pub fn from_facts(address: &Address, f: &BTreeMap<String, Value>) -> Result<Self, String> {
        let population = match f.get("population") {
            Some(Value::Text(s)) => s.clone(),
            _ => return Err("missing population".into()),
        };
        Ok(SystemBirth {
            address: address.clone(),
            population,
            mass_msun: num(f, "budget.birth_mass")? * MASS_QUANTUM_KG / MSUN_KG,
            birth_gyr: num(f, "birth_time_s")? / GYR_S,
            feh: num(f, "feh")?,
            alpha_fe: num(f, "alpha_fe")?,
            c_o: num(f, "c_o")?,
            mg_si: num(f, "mg_si")?,
            r_process_rel: num(f, "r_process_rel")?,
            k40_rel: num(f, "k40_rel")?,
            al26_rel: num(f, "al26_rel")?,
            cluster_mass_msun: num(f, "cluster_mass_msun")?,
            cluster_fuv_g0: num(f, "cluster_fuv_g0")?,
        })
    }

    /// Age at universe time `t_gyr`.
    pub fn age_gyr(&self, t_gyr: f64) -> f64 {
        t_gyr - self.birth_gyr
    }
}
