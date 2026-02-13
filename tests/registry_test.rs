use temci::utils::registry::{Registry, TypedRegistry};

#[derive(Debug, Clone, PartialEq)]
struct TestPlugin {
    name: String,
    value: i32,
}

// Test basic Registry functionality
#[test]
fn test_registry_register_and_get() {
    let mut registry = Registry::new();

    registry.register("plugin1".to_string(), Box::new(42i32));
    registry.register("plugin2".to_string(), Box::new("hello".to_string()));

    let val1 = registry.get::<i32>("plugin1");
    assert!(val1.is_some());
    assert_eq!(*val1.unwrap(), 42);

    let val2 = registry.get::<String>("plugin2");
    assert!(val2.is_some());
    assert_eq!(*val2.unwrap(), "hello");
}

#[test]
fn test_registry_get_nonexistent() {
    let registry = Registry::new();
    let val = registry.get::<i32>("nonexistent");
    assert!(val.is_none());
}

#[test]
fn test_registry_contains() {
    let mut registry = Registry::new();
    registry.register("key1".to_string(), Box::new(42i32));

    assert!(registry.contains("key1"));
    assert!(!registry.contains("key2"));
}

#[test]
fn test_registry_names() {
    let mut registry = Registry::new();
    registry.register("a".to_string(), Box::new(1i32));
    registry.register("b".to_string(), Box::new(2i32));
    registry.register("c".to_string(), Box::new(3i32));

    let names = registry.names();
    assert_eq!(names.len(), 3);
    assert!(names.contains(&"a".to_string()));
    assert!(names.contains(&"b".to_string()));
    assert!(names.contains(&"c".to_string()));
}

#[test]
fn test_registry_count() {
    let mut registry = Registry::new();
    assert_eq!(registry.count(), 0);

    registry.register("a".to_string(), Box::new(1i32));
    assert_eq!(registry.count(), 1);

    registry.register("b".to_string(), Box::new(2i32));
    assert_eq!(registry.count(), 2);
}

#[test]
fn test_registry_remove() {
    let mut registry = Registry::new();
    registry.register("key".to_string(), Box::new(42i32));

    assert!(registry.contains("key"));

    let removed = registry.remove("key");
    assert!(removed.is_some());
    assert!(!registry.contains("key"));
}

#[test]
fn test_registry_remove_nonexistent() {
    let mut registry = Registry::new();
    let removed = registry.remove("nonexistent");
    assert!(removed.is_none());
}

// Test TypedRegistry functionality
#[test]
fn test_typed_registry_register_and_get() {
    let mut registry = TypedRegistry::<TestPlugin>::new();

    let plugin1 = TestPlugin {
        name: "plugin1".to_string(),
        value: 100,
    };
    let plugin2 = TestPlugin {
        name: "plugin2".to_string(),
        value: 200,
    };

    registry.register("plugin1".to_string(), plugin1.clone());
    registry.register("plugin2".to_string(), plugin2.clone());

    let retrieved1 = registry.get("plugin1");
    assert!(retrieved1.is_some());
    assert_eq!(retrieved1.unwrap(), &plugin1);

    let retrieved2 = registry.get("plugin2");
    assert!(retrieved2.is_some());
    assert_eq!(retrieved2.unwrap(), &plugin2);
}

#[test]
fn test_typed_registry_get_mut() {
    let mut registry = TypedRegistry::<TestPlugin>::new();

    registry.register(
        "plugin1".to_string(),
        TestPlugin {
            name: "plugin1".to_string(),
            value: 100,
        },
    );

    if let Some(plugin) = registry.get_mut("plugin1") {
        plugin.value = 200;
    }

    let retrieved = registry.get("plugin1");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().value, 200);
}

#[test]
fn test_typed_registry_take() {
    let mut registry = TypedRegistry::<TestPlugin>::new();

    let plugin = TestPlugin {
        name: "plugin1".to_string(),
        value: 100,
    };
    registry.register("plugin1".to_string(), plugin.clone());

    assert!(registry.contains("plugin1"));

    let taken = registry.take("plugin1");
    assert!(taken.is_some());
    assert_eq!(taken.unwrap(), plugin);
    assert!(!registry.contains("plugin1"));
}

#[test]
fn test_typed_registry_iter() {
    let mut registry = TypedRegistry::<TestPlugin>::new();

    registry.register(
        "a".to_string(),
        TestPlugin {
            name: "a".to_string(),
            value: 1,
        },
    );
    registry.register(
        "b".to_string(),
        TestPlugin {
            name: "b".to_string(),
            value: 2,
        },
    );
    registry.register(
        "c".to_string(),
        TestPlugin {
            name: "c".to_string(),
            value: 3,
        },
    );

    let mut sum = 0;
    for (_name, plugin) in registry.iter() {
        sum += plugin.value;
    }
    assert_eq!(sum, 6);
}

#[test]
fn test_typed_registry_iter_mut() {
    let mut registry = TypedRegistry::<TestPlugin>::new();

    registry.register(
        "a".to_string(),
        TestPlugin {
            name: "a".to_string(),
            value: 1,
        },
    );
    registry.register(
        "b".to_string(),
        TestPlugin {
            name: "b".to_string(),
            value: 2,
        },
    );

    for (_name, plugin) in registry.iter_mut() {
        plugin.value *= 10;
    }

    assert_eq!(registry.get("a").unwrap().value, 10);
    assert_eq!(registry.get("b").unwrap().value, 20);
}

#[test]
fn test_typed_registry_clone() {
    let mut registry = TypedRegistry::<TestPlugin>::new();

    registry.register(
        "plugin1".to_string(),
        TestPlugin {
            name: "plugin1".to_string(),
            value: 100,
        },
    );

    let cloned = registry.clone();
    assert_eq!(cloned.get("plugin1").unwrap().value, 100);
    assert_eq!(registry.get("plugin1").unwrap().value, 100);
}
