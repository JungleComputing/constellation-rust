///! Configurations for constellation, modify the parameters to maximize
///! performance.
use crate::context::ContextVec;
use crate::StealStrategy;

/// Configuration struct
///
/// # Members
/// * `local_steal_strategy` - StealStrategy between threads on a single node
/// * `remote_steal_strategy` - StealStrategy between nodes in Constellation
/// * `number_of_nodes` - Number of nodes used
/// * `Number_of_threads` - Number of threads on each node
/// * `debug` - Set to `true` to print debug messages
/// * `context_vec` - Vector of Context struct, used to identify what contexts
/// this node supports
/// * `time_between_steals` - Time interval between stealing/distributing work
/// amongst threads. Each time work is stolen/submitted a lock on the work
/// queue is acquired, increasing this timer would make that less frequent.
#[derive(Clone)]
pub struct ConstellationConfiguration {
    pub local_steal_strategy: StealStrategy,
    pub remote_steal_strategy: StealStrategy,
    pub number_of_nodes: i32,
    pub number_of_threads: i32,
    pub debug: bool,
    pub context_vec: ContextVec,
    pub time_between_steals: u64,
}

impl ConstellationConfiguration {
    /// Create a new configuration from scratch
    ///
    /// # Arguments
    /// * `lss` - Set local steal strategy
    /// * `rss` - Set remote steal strategy
    /// * `nodes` - Number of nodes used
    /// * `threads` - Number of threads
    /// * `debug` - boolean indicating whether to print debug messages or not
    /// * `context_vec` - A vector of Context struct, indicating what activities
    /// to execute on this node
    /// * `time_between_steals` - For thread distributing activities, time it
    /// will sleep before checking for new work/stealing new work
    ///
    /// # Returns
    /// * `Box<ConstellationConfiguration>` - A boxed ConstellationConfiguration
    /// struct
    pub fn new(
        lss: StealStrategy,
        rss: StealStrategy,
        nodes: i32,
        threads: i32,
        debug: bool,
        context_vec: ContextVec,
        time_between_steals: u64,
    ) -> Box<ConstellationConfiguration> {
        //---------------------SET LOGGING--------------------------
        if debug {
            simple_logger::init().unwrap();
        }
        //----------------------------------------------------------

        Box::from(ConstellationConfiguration {
            local_steal_strategy: lss,
            remote_steal_strategy: rss,
            number_of_nodes: nodes,
            number_of_threads: threads,
            debug,
            context_vec,
            time_between_steals,
        })
    }

    /// Create a new configuration for a single threaded constellation instance
    ///
    /// # Arguments
    /// * `lss` - Set local steal strategy
    /// * `rss` - Set remote steal strategy
    /// * `nodes` - Number of nodes used
    /// * `debug` - boolean indicating whether to print debug messages or not
    /// * `context_vec` - A vector of Context struct, indicating what activities
    /// to execute on this node
    ///
    /// # Returns
    /// * `Box<ConstellationConfiguration>` - A boxed ConstellationConfiguration
    /// struct
    pub fn new_single_threaded(
        lss: StealStrategy,
        rss: StealStrategy,
        nodes: i32,
        debug: bool,
        context_vec: ContextVec,
        time_between_steals: u64,
    ) -> Box<ConstellationConfiguration> {
        ConstellationConfiguration::new(lss, rss, nodes, 1, debug, context_vec, time_between_steals)
    }
}
