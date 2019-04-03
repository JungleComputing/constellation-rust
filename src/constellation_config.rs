pub struct ConstellationConfiguration {
    //    steal_from: StealPool
    //    belongs_to: StealPool
    pub local_steal_strategy: usize,
    pub constellation_steal_strategy: usize,
    pub remote_steal_strategy: usize,
    pub number_of_nodes: i32,
    pub debug: bool,
}

impl ConstellationConfiguration {
    pub fn new(
        lss: usize,
        css: usize,
        rss: usize,
        nodes: i32,
        debug: bool,
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
            debug,
        })
    }
}
