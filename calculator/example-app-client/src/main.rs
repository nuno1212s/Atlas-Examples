use atlas_client::client;
use atlas_client::client::{Client, ClientConfig};
use atlas_client::client::unordered_client::UnorderedClientMode;
use atlas_client::concurrent_client::ConcurrentClient;
use atlas_comm_mio::{ByteStubType, MIOTCPNode};
use atlas_common::async_runtime;
use atlas_communication::{NodeInputStub, NodeStubController};
use atlas_core::ordering_protocol::OrderProtocolTolerance;
use atlas_core::serialize::NoProtocol;
use atlas_default_configs::{get_network_configurations, get_reconfig_config};
use atlas_default_configs::crypto::{FlattenedPathConstructor, FolderPathConstructor};
use atlas_reconfiguration::message::{ReconfData};
use atlas_reconfiguration::network_reconfig::NetworkInfo;
use atlas_reconfiguration::ReconfigurableNodeProtocolHandle;
use atlas_smr_core::networking::client::{CLINodeWrapper, SMRClientNetworkNode};
use atlas_smr_core::serialize::SMRSysMsg;
use example_app::app::messages::AppData;

pub type ReconfigurationMessage = ReconfData;
pub type CLIIncomingStub = NodeInputStub<ReconfigurationMessage, NoProtocol, NoProtocol, SMRSysMsg<AppData>>;
pub type CLIStubController = NodeStubController<NetworkInfo, ByteStubType, ReconfigurationMessage, NoProtocol, NoProtocol, SMRSysMsg<AppData>>;

pub type CLIByteNetworkLayer = MIOTCPNode<NetworkInfo, CLIIncomingStub, CLIStubController>;

pub type ClientNode = CLINodeWrapper<ByteStubType, CLIByteNetworkLayer, NetworkInfo, ReconfigurationMessage, AppData>;

pub type ClientNetwork = <ClientNode as SMRClientNetworkNode<NetworkInfo, ReconfigurationMessage, AppData>>::AppNode;


/// Set up the protocols with the types that have been built up to here
pub type ReconfProtocol = ReconfigurableNodeProtocolHandle;
pub type ExampleClient = Client<ReconfProtocol, AppData, ClientNetwork>;

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
    let reconfig_config = get_reconfig_config::<FolderPathConstructor>(None).unwrap();

    let node_id = reconfig_config.node_id;

    let (network_conf, pool_config) = get_network_configurations(node_id).unwrap();

    let client_cfg = ClientConfig {
        unordered_rq_mode: UnorderedClientMode::BFT,
        node: network_conf,
        reconfiguration: reconfig_config,
    };

    let client = async_runtime::block_on(client::bootstrap_client::<ReconfProtocol, AppData, ClientNode, BFT>(node_id, client_cfg)).unwrap();

    // Initialize a concurrent client from the existing client
    let concurrent_client = ConcurrentClient::from_client(client, 10).unwrap();
}
