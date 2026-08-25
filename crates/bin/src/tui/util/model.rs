use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Provider {
    pub id: String,
    pub models: HashMap<String, Model>,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub id: String,
    pub name: String,
}

pub fn parse(value: &str) -> (String, String) {
    let mut parts = value.splitn(2, '/');
    let provider_id = parts.next().unwrap_or("").to_string();
    let model_id = parts.next().unwrap_or("").to_string();
    (provider_id, model_id)
}

pub fn index(list: Option<&[Provider]>) -> HashMap<String, Provider> {
    list.unwrap_or(&[])
        .iter()
        .map(|p| (p.id.clone(), p.clone()))
        .collect()
}

pub fn get<'a>(
    list: Option<&'a [Provider]>,
    map: Option<&'a HashMap<String, Provider>>,
    provider_id: &str,
    model_id: &str,
) -> Option<&'a Model> {
    let provider = if let Some(m) = map {
        m.get(provider_id)
    } else if let Some(l) = list {
        l.iter().find(|p| p.id == provider_id)
    } else {
        None
    }?;
    provider.models.get(model_id)
}

pub fn name(
    list: Option<&[Provider]>,
    map: Option<&HashMap<String, Provider>>,
    provider_id: &str,
    model_id: &str,
) -> String {
    get(list, map, provider_id, model_id)
        .map(|m| m.name.clone())
        .unwrap_or_else(|| model_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_provider(id: &str, model_id: &str, model_name: &str) -> Provider {
        let mut models = HashMap::new();
        models.insert(
            model_id.to_string(),
            Model {
                id: model_id.to_string(),
                name: model_name.to_string(),
            },
        );
        Provider {
            id: id.to_string(),
            models,
        }
    }

    #[test]
    fn test_parse() {
        let (pid, mid) = parse("anthropic/claude-3");
        assert_eq!(pid, "anthropic");
        assert_eq!(mid, "claude-3");
    }

    #[test]
    fn test_parse_no_slash() {
        let (pid, mid) = parse("anthropic");
        assert_eq!(pid, "anthropic");
        assert_eq!(mid, "");
    }

    #[test]
    fn test_name_from_list() {
        let providers = vec![make_provider("openai", "gpt-4", "GPT-4")];
        assert_eq!(name(Some(&providers), None, "openai", "gpt-4"), "GPT-4");
    }

    #[test]
    fn test_name_fallback() {
        assert_eq!(name(None, None, "unknown", "model-id"), "model-id");
    }

    #[test]
    fn test_index() {
        let providers = vec![make_provider("openai", "gpt-4", "GPT-4")];
        let map = index(Some(&providers));
        assert!(map.contains_key("openai"));
    }
}
