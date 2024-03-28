use std::ffi::OsString;
use std::path::PathBuf;
use anyhow::anyhow;
use clap::Parser;
use config::File;
use config::FileFormat::Toml;
use atlas_common::async_runtime;
use atlas_common::ordering::SeqNo;
use atlas_common::error::*;
use atlas_decision_log::config::DecLogConfig;
use atlas_decision_log::Log;
use atlas_decision_log::serialize::LogSerialization;
use atlas_log_transfer::CollabLogTransfer;
use atlas_log_transfer::config::LogTransferConfig;
use atlas_log_transfer::messages::serialize::LTMsg;
use atlas_persistent_log::stateful_logs::monolithic_state::MonStatePersistentLog;
use atlas_reconfiguration::config::ReconfigurableNetworkConfig;
use atlas_reconfiguration::message::ReconfData;
use atlas_reconfiguration::network_reconfig::NetworkInfo;
use atlas_smr_execution::SingleThreadedMonExecutor;
use atlas_smr_replica::config::{MonolithicStateReplicaConfig, ReplicaConfig};
use atlas_smr_replica::server::monolithic_server::MonReplica;
use atlas_view_transfer::config::ViewTransferConfig;
use atlas_view_transfer::message::serialize::ViewTransfer;
use atlas_view_transfer::SimpleViewTransferProtocol;
use example_app::app::App;
use example_app::app::messages::AppData;
use example_app::state::CalculatorState;
use febft_pbft_consensus::bft::config::PBFTConfig;
use febft_pbft_consensus::bft::message::serialize::PBFTConsensus;
use febft_pbft_consensus::bft::PBFTOrderProtocol;
use febft_state_transfer::CollabStateTransfer;
use febft_state_transfer::config::StateTransferConfig;
use febft_state_transfer::message::serialize::CSTMsg;
use log::error;
use atlas_comm_mio::{ByteStubType, MIOTCPNode};
use atlas_comm_mio::config::MIOConfig;
use atlas_communication::{NodeInputStub, NodeStubController};
use atlas_default_configs::{get_network_configurations, get_reconfig_config};
use atlas_reconfiguration::ReconfigurableNodeProtocolHandle;
use atlas_smr_core::networking::{ReplicaNodeWrapper, SMRReplicaNetworkNode};
use atlas_smr_core::serialize::{Service, SMRSysMsg, StateSys};
use atlas_smr_core::SMRReq;
use atlas_smr_replica::server::Exec;
use crate::settings::ReplicaArgs;

mod settings;


/// If you want to use the default configurations,
/// just change these types to whichever types you want to use
pub type State = CalculatorState;
pub type ApplicationData = AppData;
pub type Application = App;

/// Set up the data handles so we initialize the networking layer
pub type ReconfigurationMessage = ReconfData;

/// In the case of SMR messages, we want the type that is going to be ordered to include just the actual
/// SMR Ordered Request Type, so we can use the same type for the ordering protocol
/// This type, for SMR is [atlas_smr_core::serialize::SMRReq]
///
/// These protocols are only going to be used for the ordered requests, so they only have to know about the ordered requests
/// In further parts, we can utilize [MicrobenchmarkData] directly as it requires a [D: ApplicationData], instead of just [SerType]
pub type OrderProtocolMessage = PBFTConsensus<SMRReq<ApplicationData>>;
pub type DecLogMsg = LogSerialization<SMRReq<ApplicationData>, OrderProtocolMessage, OrderProtocolMessage>;
pub type LogTransferMessage = LTMsg<SMRReq<ApplicationData>, OrderProtocolMessage, OrderProtocolMessage, DecLogMsg>;
pub type ViewTransferMessage = ViewTransfer<OrderProtocolMessage>;

/// The state transfer also requires wrapping in order to keep the [atlas_communication::serialization::Serializable] type
/// out of the state transfer protocol (and all others for that matter) for further flexibility
/// Therefore, we have to wrap the [atlas_smr_core::serialize::StateSys] type in order to get the [atlas_communication::serialization::Serializable] trait
///
pub type StateTransferMessage = CSTMsg<State>;
pub type SerStateTransferMessage = StateSys<StateTransferMessage>;


/// This type is the protocol type responsible for all SMR messages including unordered ones, so it already knows about [atlas_smr_application::ApplicationData]
pub type ProtocolDataType = Service<ApplicationData, OrderProtocolMessage, LogTransferMessage, ViewTransferMessage>;

/// Set up the networking layer with the data handles we have
///
/// In the networking level, we utilize the type which wraps [atlas_smr_application::ApplicationData]
/// and provides the [atlas_communication::serialization::Serializable] type required
/// for the network layer.
///
/// For that, we use [atlas_smr_core::serialize::SMRSysMsg]

/// Replica stub things
pub type IncomingStub = NodeInputStub<ReconfigurationMessage, ProtocolDataType, SerStateTransferMessage, SMRSysMsg<ApplicationData>>;
pub type StubController = NodeStubController<NetworkInfo, ByteStubType, ReconfigurationMessage, ProtocolDataType, SerStateTransferMessage, SMRSysMsg<ApplicationData>>;

pub type ByteNetworkLayer = MIOTCPNode<NetworkInfo, IncomingStub, StubController>;

pub type ReplicaNode = ReplicaNodeWrapper<ByteStubType, ByteNetworkLayer, NetworkInfo, ReconfigurationMessage, ApplicationData, OrderProtocolMessage,
    LogTransferMessage, ViewTransferMessage, StateTransferMessage>;

pub type ProtocolNetwork = <ReplicaNode as SMRReplicaNetworkNode<NetworkInfo, ReconfigurationMessage, ApplicationData, OrderProtocolMessage,
    LogTransferMessage, ViewTransferMessage, StateTransferMessage>>::ProtocolNode;

pub type StateTransferNetwork = <ReplicaNode as SMRReplicaNetworkNode<NetworkInfo, ReconfigurationMessage, ApplicationData, OrderProtocolMessage,
    LogTransferMessage, ViewTransferMessage, StateTransferMessage>>::StateTransferNode;

pub type AppNetwork = <ReplicaNode as SMRReplicaNetworkNode<NetworkInfo, ReconfigurationMessage, ApplicationData, OrderProtocolMessage,
    LogTransferMessage, ViewTransferMessage, StateTransferMessage>>::ApplicationNode;

pub type ReconfigurationNode = <ReplicaNode as SMRReplicaNetworkNode<NetworkInfo, ReconfigurationMessage, ApplicationData, OrderProtocolMessage,
    LogTransferMessage, ViewTransferMessage, StateTransferMessage>>::ReconfigurationNode;

/// Set up the persistent logging type with the existing data handles
pub type Logging = MonStatePersistentLog<State, ApplicationData, OrderProtocolMessage, OrderProtocolMessage, DecLogMsg, StateTransferMessage>;

/// Set up the protocols with the types that have been built up to here
pub type ReconfProtocol = ReconfigurableNodeProtocolHandle;
pub type OrderProtocol = PBFTOrderProtocol<SMRReq<ApplicationData>, ProtocolNetwork>;
pub type DecisionLog = Log<SMRReq<ApplicationData>, OrderProtocol, Logging, Exec<ApplicationData>>;
pub type LogTransferProtocol = CollabLogTransfer<SMRReq<ApplicationData>, OrderProtocol, DecisionLog, ProtocolNetwork, Logging, Exec<ApplicationData>>;
pub type ViewTransferProt = SimpleViewTransferProtocol<OrderProtocol, ProtocolNetwork>;
pub type StateTransferProtocol = CollabStateTransfer<State, StateTransferNetwork, Logging>;



pub type SMRReplica = MonReplica<ReconfProtocol, SingleThreadedMonExecutor, State, Application,
    OrderProtocol, DecisionLog, StateTransferProtocol, LogTransferProtocol,
    ViewTransferProt, ReplicaNode, Logging>;

pub type ReplicaConf = ReplicaConfig::<ReconfProtocol, State, ApplicationData, OrderProtocol, DecisionLog,
    StateTransferProtocol, LogTransferProtocol, ViewTransferProt, ReplicaNode, Logging>;

pub type MonConfig = MonolithicStateReplicaConfig::<ReconfProtocol, State, Application, OrderProtocol, DecisionLog,
    StateTransferProtocol, LogTransferProtocol, ViewTransferProt, ReplicaNode, Logging>;

pub fn init_replica_config(reconf: ReconfigurableNetworkConfig, network: MIOConfig,
                           order_protocol_config: PBFTConfig, log_transfer_config: LogTransferConfig,
                           dec_log_config: DecLogConfig, view_transfer_config: ViewTransferConfig,
                           db_path: PathBuf)
                           -> Result<ReplicaConf> {
    let db_path = db_path.into_os_string().into_string();

    let db_path = match db_path {
        Ok(db) => db,
        Err(_) => {
            return Err(anyhow!("Failed to parse persistent log folder"));
        }
    };

    let conf = ReplicaConf {
        node: network,
        next_consensus_seq: SeqNo::ZERO,
        op_config: order_protocol_config,
        dl_config: dec_log_config,
        lt_config: log_transfer_config,
        db_path,
        pl_config: (),
        reconfig_node: reconf,
        vt_config: view_transfer_config,
        p: Default::default(),
    };

    Ok(conf)
}


pub fn init_mon_replica_conf(replica_conf: ReplicaConf,
                             state_transfer_config: StateTransferConfig,
                             service: Application) -> Result<MonConfig> {
    Ok(MonConfig {
        service,
        replica_config: replica_conf,
        st_config: state_transfer_config,
    })
}


fn main() {
    let replica_args = ReplicaArgs::parse();

    let reconfiguration_cfg = get_reconfig_config().unwrap();

    let (network_cfg, pool_config) = get_network_configurations(reconfiguration_cfg.node_id).unwrap();

    // Read all configs from the corresponding files, then create the replica config, then create the MonConfig
    let config = settings::parse_febft_conf(File::new("config/febft.toml", Toml)).unwrap();

    let dec_log_config = settings::parse_dec_log_conf(File::new("config/dec_log.toml", Toml)).unwrap();

    let log_transfer = settings::parse_log_transfer_conf(File::new("config/log_transfer.toml", Toml)).unwrap();

    let state_transfer = settings::parse_state_transfer_conf(File::new("config/state_transfer.toml", Toml)).unwrap();

    let view_transfer = settings::parse_view_transfer_conf(File::new("config/view_transfer.toml", Toml)).unwrap();

    let replica_config = init_replica_config(reconfiguration_cfg, network_cfg, config,
                                             log_transfer, dec_log_config, view_transfer,
                                             replica_args.db_path).unwrap();

    let mon_config = init_mon_replica_conf(replica_config, state_transfer,
                                           Application::init()).unwrap();

    let mut replica : SMRReplica = async_runtime::block_on(MonReplica::bootstrap(mon_config)).unwrap();

    loop {
        if let Err(err) = replica.run() {
            error!("Error while executing replica {}", err);
        }
    }
}
