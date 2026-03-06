use std::collections::HashMap;

pub(super) fn match_pattern(pattern: &str, req_path: &str) -> Option<HashMap<String, String>> {
    let p = pattern.trim_matches('/');
    let r = req_path.trim_matches('/');

    let p_segs: Vec<&str> = if p.is_empty() {
        vec![]
    } else {
        p.split('/').collect()
    };
    let r_segs: Vec<&str> = if r.is_empty() {
        vec![]
    } else {
        r.split('/').collect()
    };

    let mut out = HashMap::new();

    let catch_all_name = p_segs
        .last()
        .and_then(|s| s.strip_prefix('*'))
        .filter(|s| !s.is_empty());

    if let Some(name) = catch_all_name {
        let prefix_len = p_segs.len().saturating_sub(1);
        if r_segs.len() < prefix_len {
            return None;
        }

        for (ps, rs) in p_segs[..prefix_len].iter().zip(r_segs.iter()) {
            if let Some(param_name) = ps.strip_prefix(':') {
                if param_name.is_empty() {
                    return None;
                }
                out.insert(param_name.to_string(), (*rs).to_string());
                continue;
            }

            if ps != rs {
                return None;
            }
        }

        let rest = r_segs[prefix_len..].join("/");
        out.insert(name.to_string(), rest);
        return Some(out);
    }

    if p_segs.len() != r_segs.len() {
        return None;
    }

    for (ps, rs) in p_segs.iter().zip(r_segs.iter()) {
        if let Some(name) = ps.strip_prefix(':') {
            if name.is_empty() {
                return None;
            }
            out.insert(name.to_string(), (*rs).to_string());
            continue;
        }

        if ps != rs {
            return None;
        }
    }

    Some(out)
}

pub(super) fn parse_query(query: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    if query.is_empty() {
        return out;
    }

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        if !k.is_empty() {
            out.insert(k.to_string(), v.to_string());
        }
    }

    out
}
