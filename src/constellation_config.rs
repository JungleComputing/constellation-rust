use crate::context::ContextVec;

// Time for which the multi threaded constellation will sleep between checking
// for incoming events/activites from other nodes,
// suspended events or activities
const TIME_BETWEEN_CHECKS_MULTI: u64 = 100;

#[derive(Clone)]
pub struct ConstellationConfiguration {
    pub local_steal_strategy: usize,
    pub constellation_steal_strategy: usize,
    pub remote_steal_strategy: usize,
    pub number_of_nodes: i32,
    pub number_of_threads: i32,
    pub debug: bool,
    pub context_vec: ContextVec,
    pub time_between_checks: u64,
}

impl ConstellationConfiguration {
    pub fn new(
        lss: usize,
        css: usize,
        rss: usize,
        nodes: i32,
        threads: i32,
        debug: bool,
        context_vec: ContextVec
    ) -> Box<ConstellationConfiguration> {
        //---------------------SET LOGGING--------------------------
        if debug {
            simple_logger::init().unwrap();
        }
        //----------------------------------------------------------

        Box::from(ConstellationConfiguration {
            local_steal_strategy: lss,
            constellation_steal_strategy: css,
            remote_steal_strategy: rss,
            number_of_nodes: nodes,
            number_of_threads: threads,
            debug,
            context_vec,
            time_between_checks: 0, // Single threaded does not use this
        })
    }

    pub fn new_single_threaded(
        lss: usize,
        css: usize,
        rss: usize,
        nodes: i32,
        debug: bool,
        context_vec: ContextVec,
    ) -> Box<ConstellationConfiguration> {
        let mut config = ConstellationConfiguration::new(
            lss,
            css,
            rss,
            nodes,
            1,
            debug,
            context_vec
        );

        config.time_between_checks = TIME_BETWEEN_CHECKS_MULTI;

        config
    }
}
