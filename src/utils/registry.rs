//! Registry pattern for plugin system
//!
//! Provides a generic registry for storing and retrieving items by name.
//! Includes both a type-erased Registry and a TypedRegistry for type-safe access.

use std::any::Any;
use std::collections::HashMap;

/// A type-erased registry that can store values of any type.
#[derive(Debug, Default)]
pub struct Registry {
    items: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl Registry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /// Register a value with the given name.
    pub fn register(&mut self, name: String, value: Box<dyn Any + Send + Sync>) {
        self.items.insert(name, value);
    }

    /// Get a reference to a value of type T by name.
    pub fn get<T: 'static>(&self, name: &str) -> Option<&T> {
        self.items
            .get(name)
            .and_then(|v| v.downcast_ref::<T>())
    }

    /// Get a mutable reference to a value of type T by name.
    pub fn get_mut<T: 'static>(&mut self, name: &str) -> Option<&mut T> {
        self.items
            .get_mut(name)
            .and_then(|v| v.downcast_mut::<T>())
    }

    /// Check if a value with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.items.contains_key(name)
    }

    /// Get all registered names.
    pub fn names(&self) -> Vec<String> {
        self.items.keys().cloned().collect()
    }

    /// Get the count of registered items.
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Remove and return a value by name.
    pub fn remove(&mut self, name: &str) -> Option<Box<dyn Any + Send + Sync>> {
        self.items.remove(name)
    }

    /// Clear all registered items.
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl Clone for Registry {
    fn clone(&self) -> Self {
        // Type-erased items can't be cloned directly, so we create an empty registry
        Self::new()
    }
}

/// A type-safe registry for values of a specific type.
#[derive(Debug, Clone)]
pub struct TypedRegistry<T> {
    items: HashMap<String, T>,
}

impl<T> Default for TypedRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TypedRegistry<T> {
    /// Create a new empty typed registry.
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /// Register a value with the given name.
    pub fn register(&mut self, name: String, value: T) {
        self.items.insert(name, value);
    }

    /// Get a reference to a value by name.
    pub fn get(&self, name: &str) -> Option<&T> {
        self.items.get(name)
    }

    /// Get a mutable reference to a value by name.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut T> {
        self.items.get_mut(name)
    }

    /// Check if a value with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.items.contains_key(name)
    }

    /// Get all registered names.
    pub fn names(&self) -> Vec<String> {
        self.items.keys().cloned().collect()
    }

    /// Get the count of registered items.
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Remove and return a value by name.
    pub fn remove(&mut self, name: &str) -> Option<T> {
        self.items.remove(name)
    }

    /// Take ownership of a value by name (alias for remove).
    pub fn take(&mut self, name: &str) -> Option<T> {
        self.remove(name)
    }

    /// Iterate over all items.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &T)> {
        self.items.iter()
    }

    /// Mutably iterate over all items.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut T)> {
        self.items.iter_mut()
    }

    /// Clear all registered items.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Convert into the underlying HashMap.
    pub fn into_inner(self) -> HashMap<String, T> {
        self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_basic() {
        let mut registry = Registry::new();
        registry.register("test".to_string(), Box::new(42i32));
        assert_eq!(registry.get::<i32>("test"), Some(&42));
    }

    #[test]
    fn test_typed_registry_basic() {
        let mut registry = TypedRegistry::<i32>::new();
        registry.register("test".to_string(), 42);
        assert_eq!(registry.get("test"), Some(&42));
    }
}
