/// Application thread list operations.
pub mod threads;
pub use threads::{
    Set as ThreadSet,
    Thread,
    Key as ThreadKey,
    State as ThreadState,
};

/// Interface set and operations.
pub mod interfaces;
pub use interfaces:: {
    Version,
    Key as InterfaceKey,
    Interface,
    Func as InterfaceFunc,
    InterfaceSet,
};

/// Paths to packages which contains interfaces and processes.
pub mod path;
pub use path:: {
    Path,
    RcPath,
    PathIter,
};

/// Operations on channels between threads.
pub mod channels;
pub use channels::{
    Channel,
    Key as ChannelKey,
    WaitDependency,
};

#[cfg(test)]
mod tests {
}
