mod crypto;

use std::net::SocketAddr;
use config::{Config, File};
use config::FileFormat::Toml;
use atlas_client::client::Client;
use atlas_common::node_id::{NodeId, NodeType};
use atlas_communication::mio_tcp::MIOTcpNode;
use atlas_core::serialize::ClientServiceMsg;
use atlas_reconfiguration::config::ReconfigurableNetworkConfig;
use atlas_reconfiguration::message::{NodeTriple, ReconfData};
use atlas_reconfiguration::network_reconfig::NetworkInfo;
use atlas_reconfiguration::ReconfigurableNodeProtocol;
use anyhow::Result;
use atlas_client::client::unordered_client::UnorderedClientMode;
use atlas_common::peer_addr::PeerAddr;
use example_app::app::App;
use example_app::app::messages::AppData;
use crate::settings::node_config::{get_network_config, init_client_config, Node, NodeConfig, read_node_config};

pub mod settings {
    pub mod node_config;
}

pub type Network<S> = MIOTcpNode<NetworkInfo, ReconfData, S>;

pub type ClientNetwork = Network<ClientServiceMsg<AppData>>;

/// Set up the protocols with the types that have been built up to here
pub type ReconfProtocol = ReconfigurableNodeProtocol;
pub type ExampleClient = Client<ReconfProtocol, App, ClientNetwork>;

fn setup_reconfiguration(known_nodes_config: NodeConfig) -> Result<ReconfigurableNetworkConfig> {

    let node = &known_nodes_config.own_node;

    let node_id = NodeId(node.node_id);

    let own_node = crypto::read_own_keypair(&node_id)?;

    let peer_addr = PeerAddr::new(SocketAddr::new(node.ip.parse()?, node.port), node.hostname.clone());

    let mut known_nodes = vec![];

    for bootstrap_node in known_nodes_config.bootstrap_nodes {
        let node_id = NodeId(bootstrap_node.node_id);

        let peer_addr = PeerAddr::new(SocketAddr::new(bootstrap_node.ip.parse()?, bootstrap_node.port), bootstrap_node.hostname.clone());

        known_nodes.push(NodeTriple::new(node_id, crypto::read_pk_of(&node_id)?.pk_bytes().to_vec(), peer_addr, NodeType::Replica));
    }

    Ok(ReconfigurableNetworkConfig {
        node_id,
        node_type: NodeType::Client,
        key_pair: own_node,
        our_address: peer_addr,
        known_nodes,
    })
}

fn main() {
    let node_config = File::new("config/nodes.toml", Toml).required(true);

    let node_config = read_node_config(node_config).unwrap();

    let id = NodeId(node_config.own_node.node_id);

    println!("{:?}", node_config);

    let reconfiguration_config = setup_reconfiguration(node_config).unwrap();

    let ntwrk = get_network_config(File::new("config/network.toml", Toml).required(true)).unwrap();

    let client_cfg = init_client_config(UnorderedClientMode::BFT, ntwrk, reconfiguration_config).unwrap();

    let client = ExampleClient::bootstrap::<ReconfProtocol>(id, client_cfg).unwrap();


}
