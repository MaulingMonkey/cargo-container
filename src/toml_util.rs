pub fn to_string_single_line(value: &toml::Value) -> String {
    let mut s = String::new();
    append_ssl(&mut s, value);
    s
}

fn append_ssl(o: &mut String, value: &toml::Value) {
    match value {
        toml::Value::Array(a) => {
            let mut items = a.iter();
            match items.next() {
                None => { o.push_str("[]"); return },
                Some(first) => {
                    o.push_str("[ ");
                    append_ssl(o, first);
                }
            }
            for item in items {
                o.push_str(", ");
                append_ssl(o, item);
            }
            o.push_str(" ]");
        },
        toml::Value::Table(t) => {
            let mut items = t.iter();
            match items.next() {
                None => { o.push_str("[]"); return },
                Some((k,v)) => {
                    o.push_str("{ ");
                    o.push_str(&toml::to_string(k).unwrap());
                    o.push_str(" = ");
                    append_ssl(o, v);
                }
            }
            for (k, v) in items {
                o.push_str(", ");
                o.push_str(&toml::to_string(k).unwrap());
                o.push_str(" = ");
                append_ssl(o, v);
            }
            o.push_str(" }");
        },
        other => o.push_str(&toml::to_string(other).unwrap()),
    }
}
