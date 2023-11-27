use std::ffi::OsString;
use std::path::PathBuf;
use anyhow::anyhow;
use atlas_default_configs::{get_mio_config, get_reconfig_config};
use clap::Parser;
use config::File;
use config::FileFormat::Toml;
use atlas_common::async_runtime;
use atlas_common::ordering::SeqNo;
use atlas_common::error::*;
use atlas_communication::config::MioConfig;
use atlas_communication::mio_tcp::MIOTcpNode;
use atlas_core::serialize::{Service, ServiceMessage};
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
use atlas_reconfiguration::ReconfigurableNodeProtocol;
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
use atlas_core::smr::networking::NodeWrap;
use crate::settings::ReplicaArgs;

mod settings;

pub type State = CalculatorState;
pub type ApplicationData = AppData;
pub type Application = App;

pub type ReconfigurationMessage = ReconfData;
pub type OrderProtocolMessage = PBFTConsensus<ApplicationData>;
pub type StateTransferMessage = CSTMsg<State>;
pub type DecLogMsg = LogSerialization<ApplicationData, OrderProtocolMessage, OrderProtocolMessage>;
pub type LogTransferMessage = LTMsg<ApplicationData, OrderProtocolMessage, OrderProtocolMessage, DecLogMsg>;
pub type ViewTransferMessage = ViewTransfer<OrderProtocolMessage>;

pub type Network<S> = MIOTcpNode<NetworkInfo, ReconfigurationMessage, S>;

pub type Serv = Service<ApplicationData, OrderProtocolMessage, StateTransferMessage,
    LogTransferMessage, ViewTransferMessage>;
pub type ReplicaNetwork = NodeWrap<Network<Serv>, ApplicationData, OrderProtocolMessage, StateTransferMessage,
    LogTransferMessage, ViewTransferMessage, NetworkInfo, ReconfigurationMessage>;
pub type Logging = MonStatePersistentLog<State, ApplicationData, OrderProtocolMessage, OrderProtocolMessage, DecLogMsg, StateTransferMessage>;

pub type ReconfProtocol = ReconfigurableNodeProtocol;
pub type OrderProtocol = PBFTOrderProtocol<ApplicationData, ReplicaNetwork>;
pub type DecisionLog = Log<ApplicationData, OrderProtocol, ReplicaNetwork, Logging>;
pub type LogTransferProtocol = CollabLogTransfer<ApplicationData, OrderProtocol, DecisionLog, ReplicaNetwork, Logging>;
pub type StateTransferProtocol = CollabStateTransfer<State, ReplicaNetwork, Logging>;
pub type ViewTransferProt = SimpleViewTransferProtocol<OrderProtocol, ReplicaNetwork>;
pub type SMRReplica = MonReplica<ReconfProtocol, SingleThreadedMonExecutor, State, Application,
    OrderProtocol, DecisionLog, StateTransferProtocol, LogTransferProtocol,
    ViewTransferProt, ReplicaNetwork, Logging>;

pub type ReplicaConf = ReplicaConfig::<ReconfProtocol, State, ApplicationData, OrderProtocol, DecisionLog,
    StateTransferProtocol, LogTransferProtocol, ViewTransferProt, ReplicaNetwork, Logging>;

pub type MonConfig = MonolithicStateReplicaConfig::<ReconfProtocol, State, Application, OrderProtocol, DecisionLog,
    StateTransferProtocol, LogTransferProtocol, ViewTransferProt, ReplicaNetwork, Logging>;

pub fn init_replica_config(reconf: ReconfigurableNetworkConfig, network: MioConfig,
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

    let network_cfg = get_mio_config(reconfiguration_cfg.node_id).unwrap();

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
