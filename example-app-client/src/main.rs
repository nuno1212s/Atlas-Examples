use anyhow::Result;

use atlas_client::client::{Client, ClientConfig};
use atlas_client::client::unordered_client::UnorderedClientMode;
use atlas_client::concurrent_client::ConcurrentClient;
use atlas_common::async_runtime;
use atlas_communication::config::MioConfig;
use atlas_communication::mio_tcp::MIOTcpNode;
use atlas_core::ordering_protocol::OrderProtocolTolerance;
use atlas_core::serialize::ClientServiceMsg;
use atlas_default_configs::{get_mio_config, get_reconfig_config};
use atlas_reconfiguration::config::ReconfigurableNetworkConfig;
use atlas_reconfiguration::message::ReconfData;
use atlas_reconfiguration::network_reconfig::NetworkInfo;
use atlas_reconfiguration::ReconfigurableNodeProtocol;
use example_app::app::messages::AppData;

pub type Network<S> = MIOTcpNode<NetworkInfo, ReconfData, S>;

pub type ClientNetwork = Network<ClientServiceMsg<AppData>>;

/// Set up the protocols with the types that have been built up to here
pub type ReconfProtocol = ReconfigurableNodeProtocol;
pub type ExampleClient = Client<ReconfProtocol, AppData, ClientNetwork>;

pub fn init_client_config(client_mode: UnorderedClientMode, network_config: MioConfig, reconfig: ReconfigurableNetworkConfig) -> Result<ClientConfig<ReconfProtocol, AppData, ClientNetwork>> {
    Ok(ClientConfig {
        unordered_rq_mode: client_mode,
        node: network_config,
        reconfiguration: reconfig,
    })
}

pub struct BFT;

impl OrderProtocolTolerance for BFT {
    fn get_n_for_f(f: usize) -> usize {
        return 3 * f + 1;
    }

    fn get_quorum_for_n(n: usize) -> usize {
        return Self::get_f_for_n(n) * 2 + 1;
    }

    fn get_f_for_n(n: usize) -> usize {
        return (n - 1) / 3;
    }
}

fn main() {
    let reconfig_config = get_reconfig_config().unwrap();

    let node_id = reconfig_config.node_id;

    let network_conf = get_mio_config(node_id).unwrap();

    let client_cfg = init_client_config(UnorderedClientMode::BFT, network_conf, reconfig_config).unwrap();

    let client = async_runtime::block_on(ExampleClient::bootstrap::<BFT>(node_id, client_cfg)).unwrap();

    // Initialize a concurrent client from the existing client
    let concurrent_client = ConcurrentClient::from_client(client, 10).unwrap();
}
