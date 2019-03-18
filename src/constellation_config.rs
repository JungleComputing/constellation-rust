use super::steal_strategy;

pub struct ConstellationConfiguration {
    //    context: Context,
    //    steal_from: StealPool
    //    belongs_to: StealPool
    pub local_steal_strategy: usize,
    pub constellation_steal_strategy: usize,
    pub remote_steal_strategy: usize,
    pub number_of_nodes: i32,
}

impl ConstellationConfiguration {
    fn create_config(nodes: i32) -> Box<ConstellationConfiguration> {
        Box::from(ConstellationConfiguration {
            local_steal_strategy: steal_strategy::BIGGEST,
            constellation_steal_strategy: steal_strategy::BIGGEST,
            remote_steal_strategy: steal_strategy::BIGGEST,
            number_of_nodes: nodes,
        })
    }

    pub fn new_empty(nodes: i32) -> Box<ConstellationConfiguration> {
        ConstellationConfiguration::create_config(nodes)
    }

    pub fn new_all(
        lss: usize,
        css: usize,
        rss: usize,
        nodes: i32,
    ) -> Box<ConstellationConfiguration> {
        Box::from(ConstellationConfiguration {
            local_steal_strategy: lss,
            constellation_steal_strategy: css,
            remote_steal_strategy: rss,
            number_of_nodes: nodes,
        })
    }
}
