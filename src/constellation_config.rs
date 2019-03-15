use super::steal_strategy;

pub struct ConstellationConfiguration {
//    context: Context,
//    steal_from: StealPool
//    belongs_to: StealPool
    local_steal_strategy: usize,
    constellation_steal_strategy: usize,
    remote_steal_strategy: usize,
    number_of_nodes: usize,
}

impl ConstellationConfiguration {
    fn create_config() -> Box<ConstellationConfiguration> {
        Box::from(ConstellationConfiguration {
                local_steal_strategy: steal_strategy::BIGGEST,
                constellation_steal_strategy: steal_strategy::BIGGEST,
                remote_steal_strategy: steal_strategy::BIGGEST,
                number_of_nodes: 1,
            }
        )
    }

    pub fn new_empty() -> Box<ConstellationConfiguration> {
        ConstellationConfiguration::create_config()
    }

    pub fn new_nodes(nodes: usize) -> Box<ConstellationConfiguration> {
        let mut conf = ConstellationConfiguration::create_config();

        conf.number_of_nodes = nodes;

        conf
    }

    pub fn new_all(lss: usize, css: usize, rss: usize, nodes: usize) -> Box<ConstellationConfiguration> {
        Box::from(ConstellationConfiguration {
            local_steal_strategy: lss,
            constellation_steal_strategy: css,
            remote_steal_strategy: rss,
            number_of_nodes: nodes, //TODO this should not be static
        })
    }
}