use itertools;
use liquid_error::{Error, Result};
use liquid_value::{Object, ObjectView, Scalar, ScalarCow, Value, ValueCow, ValueView};

#[derive(Clone, Default, Debug)]
struct Frame {
    name: Option<kstring::KString>,
    data: Object,
}

impl Frame {
    fn new() -> Self {
        Default::default()
    }

    fn with_name<S: Into<kstring::KString>>(name: S) -> Self {
        Self {
            name: Some(name.into()),
            data: Object::new(),
        }
    }
}

/// Stack of variables.
#[derive(Debug, Clone)]
pub struct Stack<'g> {
    globals: Option<&'g dyn ObjectView>,
    stack: Vec<Frame>,
    // State of variables created through increment or decrement tags.
    indexes: Object,
}

impl<'g> Stack<'g> {
    /// Create an empty stack
    pub fn empty() -> Self {
        Self {
            globals: None,
            indexes: Object::new(),
            // Mutable frame for globals.
            stack: vec![Frame::new()],
        }
    }

    /// Create a stack initialized with read-only `ObjectView`.
    pub fn with_globals(globals: &'g dyn ObjectView) -> Self {
        let mut stack = Self::empty();
        stack.globals = Some(globals);
        stack
    }

    /// Creates a new variable scope chained to a parent scope.
    pub(crate) fn push_frame(&mut self) {
        self.stack.push(Frame::new());
    }

    /// Creates a new variable scope chained to a parent scope.
    pub(crate) fn push_named_frame<S: Into<kstring::KString>>(&mut self, name: S) {
        self.stack.push(Frame::with_name(name));
    }

    /// Removes the topmost stack frame from the local variable stack.
    ///
    /// # Panics
    ///
    /// This method will panic if popping the topmost frame results in an
    /// empty stack. Given that a runtime is created with a top-level stack
    /// frame already in place, emptying the stack should never happen in a
    /// well-formed program.
    pub(crate) fn pop_frame(&mut self) {
        if self.stack.pop().is_none() {
            panic!("Unbalanced push/pop, leaving the stack empty.")
        };
    }

    /// The name of the currently active template.
    pub fn frame_name(&self) -> Option<kstring::KStringRef<'_>> {
        self.stack
            .iter()
            .rev()
            .find_map(|f| f.name.as_ref().map(|s| s.as_ref()))
    }

    /// Recursively index into the stack.
    pub fn try_get(&self, path: &[ScalarCow<'_>]) -> Option<ValueCow<'_>> {
        let frame = self.find_path_frame(path)?;

        liquid_value::find::try_find(frame.as_value(), path)
    }

    /// Recursively index into the stack.
    pub fn get(&self, path: &[ScalarCow<'_>]) -> Result<ValueCow<'_>> {
        let frame = self.find_path_frame(path).ok_or_else(|| {
            let key = path
                .iter()
                .next()
                .cloned()
                .unwrap_or_else(|| Scalar::new("nil"));
            let globals = itertools::join(self.roots().iter(), ", ");
            Error::with_msg("Unknown variable")
                .context("requested variable", key.to_kstr())
                .context("available variables", globals)
        })?;

        liquid_value::find::find(frame.as_value(), path)
    }

    fn roots(&self) -> Vec<kstring::KStringCow<'_>> {
        let mut roots = Vec::new();
        if let Some(globals) = self.globals {
            roots.extend(globals.keys());
        }
        for frame in self.stack.iter() {
            roots.extend(frame.data.keys().map(kstring::KStringCow::from));
        }
        roots.sort();
        roots.dedup();
        roots
    }

    fn find_path_frame<'a>(&'a self, path: &[ScalarCow<'_>]) -> Option<&'a dyn ObjectView> {
        let key = path.iter().next()?;
        let key = key.to_kstr();
        self.find_frame(key.as_str())
    }

    fn find_frame<'a>(&'a self, name: &str) -> Option<&'a dyn ObjectView> {
        for frame in self.stack.iter().rev() {
            if frame.data.contains_key(name) {
                return Some(&frame.data);
            }
        }

        if self.globals.map(|g| g.contains_key(name)).unwrap_or(false) {
            return self.globals;
        }

        if self.indexes.contains_key(name) {
            return Some(&self.indexes);
        }

        None
    }

    /// Used by increment and decrement tags
    pub fn set_index<S>(&mut self, name: S, val: Value) -> Option<Value>
    where
        S: Into<kstring::KString>,
    {
        self.indexes.insert(name.into(), val)
    }

    /// Used by increment and decrement tags
    pub fn get_index<'a>(&'a self, name: &str) -> Option<&'a Value> {
        self.indexes.get(name)
    }

    /// Sets a value in the global runtime.
    pub fn set_global<S>(&mut self, name: S, val: Value) -> Option<Value>
    where
        S: Into<kstring::KString>,
    {
        let name = name.into();
        self.global_frame().insert(name, val)
    }

    /// Sets a value to the rendering runtime.
    /// Note that it needs to be wrapped in a liquid::Value.
    ///
    /// # Panics
    ///
    /// Panics if there is no frame on the local values stack. Runtime
    /// instances are created with a top-level stack frame in place, so
    /// this should never happen in a well-formed program.
    pub fn set<S>(&mut self, name: S, val: Value) -> Option<Value>
    where
        S: Into<kstring::KString>,
    {
        self.current_frame().insert(name.into(), val)
    }

    fn current_frame(&mut self) -> &mut Object {
        match self.stack.last_mut() {
            Some(frame) => &mut frame.data,
            None => panic!("Global frame removed."),
        }
    }

    fn global_frame(&mut self) -> &mut Object {
        match self.stack.first_mut() {
            Some(frame) => &mut frame.data,
            None => panic!("Global frame removed."),
        }
    }
}

impl<'g> Default for Stack<'g> {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use liquid_value::values::ValueViewCmp;

    #[test]
    fn stack_find_frame() {
        let mut stack = Stack::empty();
        stack.set_global("number", Value::scalar(42f64));
        assert!(stack.find_frame("number").is_some(),);
    }

    #[test]
    fn stack_find_frame_failure() {
        let mut stack = Stack::empty();
        let mut post = Object::new();
        post.insert("number".into(), Value::scalar(42f64));
        stack.set_global("post", Value::Object(post));
        assert!(stack.find_frame("post.number").is_none());
    }

    #[test]
    fn stack_get() {
        let mut stack = Stack::empty();
        let mut post = Object::new();
        post.insert("number".into(), Value::scalar(42f64));
        stack.set_global("post", Value::Object(post));
        let indexes = [Scalar::new("post"), Scalar::new("number")];
        assert_eq!(&stack.get(&indexes).unwrap(), &ValueViewCmp::new(&42f64));
    }
}
