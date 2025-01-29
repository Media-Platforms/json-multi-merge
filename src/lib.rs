use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use pyo3::exceptions::PyTypeError;
use pyo3::PyAny;
use serde_json::{Map, Value};

#[pyfunction(signature = (*objs))]
fn merge(objs: &PyTuple) -> PyResult<PyObject> {
    let py = objs.py();
    let mut merged = Value::Object(Map::new());

    if objs.is_empty() {
        return json_to_py_object(py, &merged);
    }

    for obj in objs.iter() {
        // Validate input is a dictionary
        if !obj.is_instance_of::<PyDict>() {
            return Err(PyTypeError::new_err("All inputs must be dictionaries"));
        }

        let current = py_object_to_json(obj)?;
        merged = merge_json_objects(&merged, &current);
    }

    json_to_py_object(py, &merged)
}

#[derive(Debug)]
enum Action {
    Omit,
    Replace,
    Merge,
}

fn process_key(key: &str) -> (String, Action) {
    if key.ends_with("--") {
        (key[..key.len()-2].to_string(), Action::Omit)
    } else if key.ends_with('!') {
        (key[..key.len()-1].to_string(), Action::Replace)
    } else {
        (key.to_string(), Action::Merge)
    }
}

fn merge_json_objects(a: &Value, b: &Value) -> Value {
    // Always process b's structure even when replacing
    match b {
        Value::Object(b_obj) => {
            let mut base = match a {
                Value::Object(a_obj) => a_obj.clone(),
                _ => Map::new(),
            };

            for (raw_key, b_val) in b_obj {
                let (base_key, action) = process_key(raw_key);

                match action {
                    Action::Omit => {
                        base.remove(&base_key);
                    },
                    Action::Replace => {
                        // Process nested modifiers in replacement
                        let replaced = merge_json_objects(&Value::Null, b_val);
                        base.insert(base_key, replaced);
                    },
                    Action::Merge => {
                        let existing = base.get(&base_key).unwrap_or(&Value::Null);
                        let merged = merge_json_objects(existing, b_val);
                        base.insert(base_key, merged);
                    }
                }
            }
            Value::Object(base)
        },
        Value::Array(_) => b.clone(),
        _ => b.clone(),
    }
}

fn py_object_to_json(obj: &PyAny) -> PyResult<Value> {
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = Map::new();
        for (key, value) in dict.iter() {
            let key_str = key.extract::<String>()?;
            let value_json = py_object_to_json(value)?;
            map.insert(key_str, value_json);
        }
        Ok(Value::Object(map))
    } else if let Ok(list) = obj.downcast::<PyList>() {
        let mut arr = Vec::new();
        for item in list.iter() {
            arr.push(py_object_to_json(item)?);
        }
        Ok(Value::Array(arr))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(Value::String(s))
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(Value::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(Value::Number(i.into()))
    } else if let Ok(f) = obj.extract::<f64>() {
        serde_json::Number::from_f64(f)
            .map(Value::Number)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid float"))
    } else if obj.is_none() {
        Ok(Value::Null)
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Unsupported type",
        ))
    }
}

fn json_to_py_object(py: Python, value: &Value) -> PyResult<PyObject> {
    Ok(match value {
        Value::Null => py.None().into(),
        Value::Bool(b) => b.into_py(py),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_py(py)
            } else if let Some(f) = n.as_f64() {
                f.into_py(py)
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Invalid number",
                ));
            }
        }
        Value::String(s) => s.into_py(py),
        Value::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                py_list.append(json_to_py_object(py, item)?)?;
            }
            py_list.into()
        }
        Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (k, v) in obj {
                let py_key = k.into_py(py);
                let py_val = json_to_py_object(py, v)?;
                py_dict.set_item(py_key, py_val)?;
            }
            py_dict.into()
        }
    })
}

#[pymodule]
fn json_multi_merge(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(merge, m)?)?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_merge() {
        let a = json!({"a": 1, "b": 2});
        let b = json!({"b": 3, "c": 4});
        let merged = merge_json_objects(&a, &b);
        assert_eq!(merged, json!({"a": 1, "b": 3, "c": 4}));
    }

    #[test]
    fn test_variadic_merge() {
        let a = json!({"a": 1});
        let b = json!({"b": 2});
        let c = json!({"c": 3});

        let merged = merge_json_objects(&merge_json_objects(&a, &b), &c);
        assert_eq!(merged, json!({"a": 1, "b": 2, "c": 3}));
    }

    #[test]
    fn test_single_argument() {
        let a = json!({"a": 1});
        let merged = merge_json_objects(&Value::Object(Map::new()), &a);
        assert_eq!(merged, a);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(merge_json_objects(&Value::Null, &Value::Null), Value::Null);
    }

    #[test]
    fn test_three_level_merge() {
        let objs = vec![
            json!({"config": {"debug": true}}),
            json!({"config": {"port": 8080}}),
            json!({"config!": {"production": true}}),
        ];
        let merged = objs.into_iter().fold(Value::Null, |acc, x| merge_json_objects(&acc, &x));
        assert_eq!(merged, json!({"config": {"production": true}}));
    }

    #[test]
    fn test_nested_merge() {
        let a = json!({
            "user": {
                "name": "Alice",
                "address": {"city": "Paris"}
            }
        });
        let b = json!({
            "user": {
                "age": 30,
                "address": {"country": "France"}
            }
        });
        let expected = json!({
            "user": {
                "name": "Alice",
                "age": 30,
                "address": {"city": "Paris", "country": "France"}
            }
        });
        assert_eq!(merge_json_objects(&a, &b), expected);
    }

    #[test]
    fn test_array_replacement() {
        let a = json!({"items": [1, 2], "tags": ["old"]});
        let b = json!({"items": [3, 4], "tags!": ["new"]});
        let merged = merge_json_objects(&a, &b);
        assert_eq!(merged["items"], json!([3, 4]));
        assert_eq!(merged["tags"], json!(["new"]));
    }

    #[test]
    fn test_omit_modifier() {
        let a = json!({"keep": 1, "remove_me": 2});
        let b = json!({"remove_me--": 3, "new_key": 4});
        let merged = merge_json_objects(&a, &b);
        assert_eq!(merged, json!({"keep": 1, "new_key": 4}));
    }

    #[test]
    fn test_replace_modifier() {
        let a = json!({
            "config": {
                "debug": false,
                "plugins": ["basic"]
            }
        });
        let b = json!({
            "config!": {
                "production": true,
                "plugins": ["advanced"]
            }
        });
        let expected = json!({
            "config": {
                "production": true,
                "plugins": ["advanced"]
            }
        });
        assert_eq!(merge_json_objects(&a, &b), expected);
    }

    #[test]
    fn test_nested_modifiers() {
        let a = json!({"user": {"name": "Alice"}});
        let b = json!({"user!": {"meta--": {"id": 123}, "role": "admin"}});
        let merged = merge_json_objects(&a, &b);
        assert_eq!(merged["user"], json!({"role": "admin"})); // meta-- removed
    }

    #[test]
    fn test_complex_modifiers() {
        let a = json!({"a": {"c": [1, 2]}});
        let b = json!({"a!": {"b": {"new": 2}, "c--": [3, 4]}});
        let merged = merge_json_objects(&a, &b);
        assert_eq!(merged["a"], json!({"b": {"new": 2}})); // c-- removed
    }

    #[test]
    fn test_deeply_nested_structures() {
        let b = json!({
            "level1": {
                "level2!": {
                    "level3": {
                        "update!": {"b": 2},
                        "new": "value"
                    }
                }
            }
        });
        let merged = merge_json_objects(&Value::Null, &b);
        assert_eq!(
            merged["level1"]["level2"]["level3"],
            json!({"update": {"b": 2}, "new": "value"})
        );
    }

    #[test]
    fn test_deeply_nested_structures_2() {
        let a = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "keep": "original",
                        "update": {"a": 1}
                    }
                }
            }
        });
        let b = json!({
            "level1": {
                "level2!": {
                    "level3": {
                        "update!": {"b": 2},
                        "new": "value"
                    }
                }
            }
        });
        let expected = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "update": {"b": 2},
                        "new": "value"
                    }
                }
            }
        });
        assert_eq!(merge_json_objects(&a, &b), expected);
    }

    #[test]
    fn test_edge_cases() {
        // Empty objects
        assert_eq!(merge_json_objects(&json!({}), &json!({})), json!({}));

        // Null handling
        let a = json!({"key": null});
        let b = json!({"key!": "value"});
        assert_eq!(merge_json_objects(&a, &b), json!({"key": "value"}));

        // Type conflicts
        let a = json!({"key": {"nested": 1}});
        let b = json!({"key": [1, 2, 3]});
        assert_eq!(merge_json_objects(&a, &b), json!({"key": [1, 2, 3]}));
    }
}
