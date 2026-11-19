use std::collections::HashMap;

use serde::Serialize;

pub fn serialize_env<S>(env: &HashMap<String, String>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  let mut res = env.clone();
  for key in env.keys().filter(|k| k.starts_with("{")) {
    res.remove(&key.clone());
  }
  res.serialize(serializer)
}

pub fn is_env_empty(env: &HashMap<String, String>) -> bool {
  env.keys().all(|k| k.starts_with("{"))
}
