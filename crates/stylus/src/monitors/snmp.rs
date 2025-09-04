use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use rasn_smi::rasn::types::{ObjectIdentifier, Oid};
use rasn_smi::ObjectType;
use serde::{Deserialize, Serialize};

use crate::{
    config::{MonitorDirAxisValue, MonitorDirChildConfig, MonitorDirTestConfig},
    expressions::{self, Value},
    interpolate::interpolate_id,
    monitor::{MonitorMessageProcessor, MonitorMessageProcessorInstance},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct SnmpNetworkMonitorConfig {
    target: SnmpNetworkMonitorSnmpConfig,
    pub id: String,
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    #[serde(default = "default_include")]
    pub include: String,
    #[serde(default = "default_exclude")]
    pub exclude: String,
    #[serde(default = "default_red")]
    pub red: String,
    #[serde(default = "default_green")]
    pub green: String,
    #[serde(skip_deserializing)]
    pub children: BTreeMap<String, MonitorDirChildConfig>,
    #[serde(skip_deserializing)]
    pub test: Option<MonitorDirTestConfig>,
}

fn default_include() -> String {
    format!("true")
}

fn default_exclude() -> String {
    format!("false")
}

fn default_red() -> String {
    format!("false")
}

fn default_green() -> String {
    format!("ifOperStatus == 'up' and ifAdminStatus == 'up'")
}

const OID_MAP: &[(&str, &Oid, &[(u32, &str)])] = &[
    ("ifIndex", rasn_mib::interfaces::Index::VALUE, &[]),
    ("ifDescr", rasn_mib::interfaces::Descr::VALUE, &[]),
    ("ifType", rasn_mib::interfaces::Type::VALUE, ENUM_MAP_TYPE),
    ("ifMtu", rasn_mib::interfaces::Mtu::VALUE, &[]),
    ("ifSpeed", rasn_mib::interfaces::Speed::VALUE, &[]),
    (
        "ifPhysAddress",
        rasn_mib::interfaces::PhysAddress::VALUE,
        &[],
    ),
    (
        "ifAdminStatus",
        rasn_mib::interfaces::AdminStatus::VALUE,
        ENUM_MAP_OPER_STATUS,
    ),
    (
        "ifOperStatus",
        rasn_mib::interfaces::OperStatus::VALUE,
        ENUM_MAP_OPER_STATUS,
    ),
    ("ifLastChange", rasn_mib::interfaces::LastChange::VALUE, &[]),
    ("ifInOctets", rasn_mib::interfaces::InOctets::VALUE, &[]),
    (
        "ifInUcastPkts",
        rasn_mib::interfaces::InUcastPkts::VALUE,
        &[],
    ),
    (
        "ifInNUcastPkts",
        rasn_mib::interfaces::InNUcastPkts::VALUE,
        &[],
    ),
    ("ifInDiscards", rasn_mib::interfaces::InDiscards::VALUE, &[]),
    ("ifInErrors", rasn_mib::interfaces::InErrors::VALUE, &[]),
    (
        "ifInUnknownProtos",
        rasn_mib::interfaces::InUnknownProtos::VALUE,
        &[],
    ),
    ("ifOutOctets", rasn_mib::interfaces::OutOctets::VALUE, &[]),
    (
        "ifOutUcastPkts",
        rasn_mib::interfaces::OutUcastPkts::VALUE,
        &[],
    ),
    (
        "ifOutNUcastPkts",
        rasn_mib::interfaces::OutNUcastPkts::VALUE,
        &[],
    ),
    (
        "ifOutDiscards",
        rasn_mib::interfaces::OutDiscards::VALUE,
        &[],
    ),
    ("ifOutErrors", rasn_mib::interfaces::OutErrors::VALUE, &[]),
    ("ifOutQLen", rasn_mib::interfaces::OutQLen::VALUE, &[]),
    ("ifSpecific", rasn_mib::interfaces::Specific::VALUE, &[]),
];

const ENUM_MAP_OPER_STATUS: &[(u32, &str)] = &[(1, "up"), (2, "down"), (3, "testing")];

const ENUM_MAP_TYPE: &[(u32, &str)] = &[
    (1, "other"),
    (2, "regular1822"),
    (3, "hdh1822"),
    (4, "ddnX25"),
    (5, "rfc877x25"),
    (6, "ethernetCsmacd"),
    (7, "iso88023Csmacd"),
    (8, "iso88024TokenBus"),
    (9, "iso88025TokenRing"),
    (10, "iso88026Man"),
    (11, "starLan"),
    (12, "proteon10Mbit"),
    (13, "proteon80Mbit"),
    (14, "hyperchannel"),
    (15, "fddi"),
    (16, "lapb"),
    (17, "sdlc"),
    (18, "ds1"),
    (19, "e1"),
    (20, "basicISDN"),
    (21, "primaryISDN"),
    (22, "propPointToPointSerial"),
    (23, "ppp"),
    (24, "softwareLoopback"),
    (25, "eon"),
    (26, "ethernet3Mbit"),
    (27, "nsip"),
    (28, "slip"),
    (29, "ultra"),
    (30, "ds3"),
    (31, "sip"),
    (32, "frameRelay"),
    (33, "rs232"),
    (34, "para"),
    (35, "arcnet"),
    (36, "arcnetPlus"),
    (37, "atm"),
    (38, "miox25"),
    (39, "sonet"),
    (40, "x25ple"),
    (41, "iso88022llc"),
    (42, "localTalk"),
    (43, "smdsDxi"),
    (44, "frameRelayService"),
    (45, "v35"),
    (46, "hssi"),
    (47, "hippi"),
    (48, "modem"),
    (49, "aal5"),
    (50, "sonetPath"),
    (51, "sonetVT"),
    (52, "smdsIcip"),
    (53, "propVirtual"),
    (54, "propMultiplexor"),
    (55, "ieee80212"),
    (56, "fibreChannel"),
    (57, "hippiInterface"),
    (58, "frameRelayInterconnect"),
    (59, "aflane8023"),
    (60, "aflane8025"),
    (61, "cctEmul"),
    (62, "fastEther"),
    (63, "isdn"),
    (64, "v11"),
    (65, "v36"),
    (66, "g703at64k"),
    (67, "g703at2mb"),
    (68, "qllc"),
    (69, "fastEtherFX"),
    (70, "channel"),
    (71, "ieee80211"),
    (72, "ibm370parChan"),
    (73, "escon"),
    (74, "dlsw"),
    (75, "isdns"),
    (76, "isdnu"),
    (77, "lapd"),
    (78, "ipSwitch"),
    (79, "rsrb"),
    (80, "atmLogical"),
    (81, "ds0"),
    (82, "ds0Bundle"),
    (83, "bsc"),
    (84, "async"),
    (85, "cnr"),
    (86, "iso88025Dtr"),
    (87, "eplrs"),
    (88, "arap"),
    (89, "propCnls"),
    (90, "hostPad"),
    (91, "termPad"),
    (92, "frameRelayMPI"),
    (93, "x213"),
    (94, "adsl"),
    (95, "radsl"),
    (96, "sdsl"),
    (97, "vdsl"),
    (98, "iso88025CRFPInt"),
    (99, "myrinet"),
    (100, "voiceEM"),
    (101, "voiceFXO"),
    (102, "voiceFXS"),
    (103, "voiceEncap"),
    (104, "voiceOverIp"),
    (105, "atmDxi"),
    (106, "atmFuni"),
    (107, "atmIma"),
    (108, "pppMultilinkBundle"),
    (109, "ipOverCdlc"),
    (110, "ipOverClaw"),
    (111, "stackToStack"),
    (112, "virtualIpAddress"),
    (113, "mpc"),
    (114, "ipOverAtm"),
    (115, "iso88025Fiber"),
    (116, "tdlc"),
    (117, "gigabitEthernet"),
    (118, "hdlc"),
    (119, "lapf"),
    (120, "v37"),
    (121, "x25mlp"),
    (122, "x25huntGroup"),
    (123, "transpHdlc"),
    (124, "interleave"),
    (125, "fast"),
    (126, "ip"),
    (127, "docsCableMaclayer"),
    (128, "docsCableDownstream"),
    (129, "docsCableUpstream"),
    (130, "a12MppSwitch"),
    (131, "tunnel"),
    (132, "coffee"),
    (133, "ces"),
    (134, "atmSubInterface"),
    (135, "l2vlan"),
    (136, "l3ipvlan"),
    (137, "l3ipxvlan"),
    (138, "digitalPowerline"),
    (139, "mediaMailOverIp"),
    (140, "dtm"),
    (141, "dcn"),
    (142, "ipForward"),
    (143, "msdsl"),
    (144, "ieee1394"),
    (145, "if-gsn"),
    (146, "dvbRccMacLayer"),
    (147, "dvbRccDownstream"),
    (148, "dvbRccUpstream"),
    (149, "atmVirtual"),
    (150, "mplsTunnel"),
    (151, "srp"),
    (152, "voiceOverAtm"),
    (153, "voiceOverFrameRelay"),
    (154, "idsl"),
    (155, "compositeLink"),
    (156, "ss7SigLink"),
    (157, "propWirelessP2P"),
    (158, "frForward"),
    (159, "rfc1483"),
    (160, "usb"),
    (161, "ieee8023adLag"),
    (162, "bgppolicyaccounting"),
    (163, "frf16MfrBundle"),
    (164, "h323Gatekeeper"),
    (165, "h323Proxy"),
    (166, "mpls"),
    (167, "mfSigLink"),
    (168, "hdsl2"),
    (169, "shdsl"),
    (170, "ds1FDL"),
    (171, "pos"),
    (172, "dvbAsiIn"),
    (173, "dvbAsiOut"),
    (174, "plc"),
    (175, "nfas"),
    (176, "tr008"),
    (177, "gr303RDT"),
    (178, "gr303IDT"),
    (179, "isup"),
    (180, "propDocsWirelessMaclayer"),
    (181, "propDocsWirelessDownstream"),
    (182, "propDocsWirelessUpstream"),
    (183, "hiperlan2"),
    (184, "propBWAp2Mp"),
    (185, "sonetOverheadChannel"),
    (186, "digitalWrapperOverheadChannel"),
    (187, "aal2"),
    (188, "radioMAC"),
    (189, "atmRadio"),
    (190, "imt"),
    (191, "mvl"),
    (192, "reachDSL"),
    (193, "frDlciEndPt"),
    (194, "atmVciEndPt"),
    (195, "opticalChannel"),
    (196, "opticalTransport"),
    (197, "propAtm"),
    (198, "voiceOverCable"),
    (199, "infiniband"),
    (200, "teLink"),
    (201, "q2931"),
    (202, "virtualTg"),
    (203, "sipTg"),
    (204, "sipSig"),
    (205, "docsCableUpstreamChannel"),
    (206, "econet"),
    (207, "pon155"),
    (208, "pon622"),
    (209, "bridge"),
    (210, "linegroup"),
    (211, "voiceEMFGD"),
    (212, "voiceFGDEANA"),
    (213, "voiceDID"),
    (214, "mpegTransport"),
    (215, "sixToFour"),
    (216, "gtp"),
    (217, "pdnEtherLoop1"),
    (218, "pdnEtherLoop2"),
    (219, "opticalChannelGroup"),
    (220, "homepna"),
    (221, "gfp"),
    (222, "ciscoISLvlan"),
    (223, "actelisMetaLOOP"),
    (224, "fcipLink"),
    (225, "rpr"),
    (226, "qam"),
    (227, "lmp"),
    (228, "cblVectaStar"),
    (229, "docsCableMCmtsDownstream"),
    (230, "adsl2"),
    (231, "macSecControlledIF"),
    (232, "macSecUncontrolledIF"),
    (233, "aviciOpticalEther"),
    (234, "atmbond"),
    (235, "voiceFGDOS"),
    (236, "mocaVersion1"),
    (237, "ieee80216WMAN"),
    (238, "adsl2plus"),
    (239, "dvbRcsMacLayer"),
    (240, "dvbTdm"),
    (241, "dvbRcsTdma"),
    (242, "x86Laps"),
    (243, "wwanPP"),
    (244, "wwanPP2"),
    (245, "voiceEBS"),
    (246, "ifPwType"),
    (247, "ilan"),
    (248, "pip"),
    (249, "aluELP"),
    (250, "gpon"),
    (251, "vdsl2"),
    (252, "capwapDot11Profile"),
    (253, "capwapDot11Bss"),
    (254, "capwapWtpVirtualRadio"),
    (255, "bits"),
    (256, "docsCableUpstreamRfPort"),
    (257, "cableDownstreamRfPort"),
    (258, "vmwareVirtualNic"),
    (259, "ieee802154"),
    (260, "otnOdu"),
    (261, "otnOtu"),
    (262, "ifVfiType"),
    (263, "g9981"),
    (264, "g9982"),
    (265, "g9983"),
    (266, "aluEpon"),
    (267, "aluEponOnu"),
    (268, "aluEponPhysicalUni"),
    (269, "aluEponLogicalLink"),
    (270, "aluGponOnu"),
    (271, "aluGponPhysicalUni"),
    (272, "vmwareNicTeam"),
    (277, "docsOfdmDownstream"),
    (278, "docsOfdmaUpstream"),
    (279, "gfast"),
    (280, "sdci"),
    (281, "xboxWireless"),
    (282, "fastdsl"),
    (283, "docsCableScte55d1FwdOob"),
    (284, "docsCableScte55d1RetOob"),
    (285, "docsCableScte55d2DsOob"),
    (286, "docsCableScte55d2UsOob"),
    (287, "docsCableNdf"),
    (288, "docsCableNdr"),
    (289, "ptm"),
    (290, "ghn"),
    (291, "otnOtsi"),
    (292, "otnOtuc"),
    (293, "otnOduc"),
    (294, "otnOtsig"),
    (295, "microwaveCarrierTermination"),
    (296, "microwaveRadioLinkTerminal"),
    (297, "ieee8021axDrni"),
    (298, "ax25"),
    (299, "ieee19061nanocom"),
    (300, "cpri"),
    (301, "omni"),
    (302, "roe"),
    (303, "p2pOverLan"),
];

impl SnmpNetworkMonitorConfig {
    pub fn test(&self) -> MonitorDirTestConfig {
        let binary = if self.target.bulk {
            "snmpbulkwalk"
        } else {
            "snmpwalk"
        };

        let mut parts: Vec<String> = vec![binary.to_string()];

        // Output formatting: -O sQ
        parts.push("-OsQfne".into());

        // Timeout (seconds)
        parts.push("-t".into());
        parts.push(self.timeout.as_secs().to_string());

        // SNMP version
        let version_flag = match self.target.version {
            1 => "1",
            2 => "2c",
            3 => "3",
            _ => "2c",
        };
        parts.push("-v".into());
        parts.push(version_flag.to_string());

        // Auth (v1/v2c vs v3)
        match self.target.version {
            1 | 2 => {
                parts.push("-c".into());
                parts.push(self.target.community.clone());
            }
            3 => {
                if let Some(ref username) = self.target.username {
                    parts.push("-u".into());
                    parts.push(username.into());
                }

                let has_auth =
                    self.target.auth_protocol.is_some() && self.target.auth_password.is_some();
                let has_priv = self.target.privacy_protocol.is_some()
                    && self.target.privacy_password.is_some();

                let level = if has_priv {
                    "authPriv"
                } else if has_auth {
                    "authNoPriv"
                } else {
                    "noAuthNoPriv"
                };
                parts.push("-l".into());
                parts.push(level.to_string());

                if has_auth {
                    parts.push("-a".into());
                    parts.push(self.target.auth_protocol.as_ref().unwrap().to_uppercase());
                    parts.push("-A".into());
                    parts.push({
                        let input: &str = self.target.auth_password.as_ref().unwrap();
                        input.into()
                    });
                }
                if has_priv {
                    parts.push("-x".into());
                    parts.push(
                        self.target
                            .privacy_protocol
                            .as_ref()
                            .unwrap()
                            .to_uppercase(),
                    );
                    parts.push("-X".into());
                    parts.push({
                        let input: &str = self.target.privacy_password.as_ref().unwrap();
                        input.into()
                    });
                }
            }
            _ => {}
        }

        // Agent (host:port)
        if let Some(port) = self.target.port {
            parts.push(format!("{}:{}", self.target.host, port));
        } else {
            parts.push(self.target.host.clone());
        }

        parts.push(rasn_mib::interfaces::Table::VALUE.to_string());

        MonitorDirTestConfig {
            interval: self.interval,
            timeout: self.timeout,
            command: PathBuf::from("/usr/bin/env"),
            args: parts,
            processor: Some(Arc::new(SnmpMonitorMessageProcessor {
                id: self.id.clone(),
                include: self.include.clone(),
                exclude: self.exclude.clone(),
                red: self.red.clone(),
                green: self.green.clone(),
            })),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct SnmpNetworkMonitorSnmpConfig {
    pub host: String,
    pub port: Option<u16>,
    #[serde(default = "default_version")]
    pub version: u8,
    #[serde(default = "default_community")]
    pub community: String,
    pub username: Option<String>,
    pub auth_protocol: Option<String>,
    pub auth_password: Option<String>,
    pub privacy_protocol: Option<String>,
    pub privacy_password: Option<String>,
    #[serde(default = "default_bulk")]
    pub bulk: bool,
}

fn default_community() -> String {
    "public".to_string()
}

fn default_version() -> u8 {
    2
}

fn default_bulk() -> bool {
    true
}

#[derive(Debug)]
pub struct SnmpMonitorMessageProcessor {
    id: String,
    include: String,
    exclude: String,
    red: String,
    green: String,
}

#[derive(Debug, Default)]
pub struct SnmpMonitorMessageProcessorInstance {
    id: String,
    include: String,
    exclude: String,
    red: String,
    green: String,
    ports: Mutex<HashMap<usize, HashMap<String, Value>>>,
}

impl MonitorMessageProcessor for SnmpMonitorMessageProcessor {
    fn new(&self) -> Box<dyn MonitorMessageProcessorInstance> {
        Box::new(SnmpMonitorMessageProcessorInstance {
            id: self.id.clone(),
            include: self.include.clone(),
            exclude: self.exclude.clone(),
            red: self.red.clone(),
            green: self.green.clone(),
            ports: Default::default(),
        })
    }
}

fn parse_indexed_oid(input: &str) -> Option<(ObjectIdentifier, u32)> {
    let Some(input) = input.strip_prefix(".") else {
        return None;
    };
    let mut arcs = input
        .split(".")
        .map(|s| s.parse::<u32>().unwrap())
        .collect::<Vec<u32>>();
    let index = arcs.pop().unwrap();
    if let Some(oid) = ObjectIdentifier::new(arcs) {
        Some((oid, index))
    } else {
        log::warn!("Failed to parse OID from: {}", input);
        None
    }
}

impl MonitorMessageProcessorInstance for SnmpMonitorMessageProcessorInstance {
    fn process_message(&self, input: &str) -> Vec<String> {
        // Parse the input as <oid>.index = <valud>?
        // Note that value may be missing and input may end in the equals. This is considered an empty string.

        if let Some((left, right)) = input.split_once("=") {
            let left = left.trim();
            let (left_oid, left_index) = if let Some((oid, index)) = parse_indexed_oid(left) {
                (oid, index)
            } else {
                log::warn!("Failed to parse OID from: {}", left);
                return vec![];
            };
            let mut right = right.trim();

            let mut left = None;
            for (name, oid, enum_values) in OID_MAP {
                if left_oid == *oid {
                    left = Some((name, oid));
                    if !enum_values.is_empty() {
                        let value = right.parse::<u32>().unwrap_or_default();
                        for (enum_value, name) in *enum_values {
                            if *enum_value == value {
                                right = name;
                                break;
                            }
                        }
                    }
                    break;
                }
            }

            let Some((oid, _)) = left else {
                log::warn!("Failed to find OID in OID_MAP: {}", left_oid);
                return vec![];
            };

            let v = if let Ok(v) = right.parse::<i64>() {
                Value::Int(v)
            } else {
                Value::Str(right.to_string())
            };

            self.ports
                .lock()
                .unwrap()
                .entry(left_index as _)
                .or_default()
                .insert(oid.to_string(), v);
            return vec![];
        };

        log::warn!("Unexpected snmpwalk/snmpbulkwalk output: {}", input);
        vec![]
    }

    fn finalize(&self) -> Vec<String> {
        let mut result = vec![];

        for (port_index, port_metadata) in std::mem::take(&mut *self.ports.lock().unwrap()) {
            let include = calculate_bool(&self.include, &port_metadata);
            if !include {
                continue;
            }

            let exclude = calculate_bool(&self.exclude, &port_metadata);
            if exclude {
                continue;
            }

            let red = calculate_bool(&self.red, &port_metadata);
            let green = calculate_bool(&self.green, &port_metadata);

            let mut values = BTreeMap::new();
            values.insert(
                "index".into(),
                MonitorDirAxisValue::Number(port_index as i64),
            );
            let Ok(port_id) = interpolate_id(&values, &self.id) else {
                log::warn!(
                    "Failed to interpolate id for port {}: {:?}",
                    port_index,
                    values
                );
                continue;
            };

            for (oid, value) in port_metadata {
                result.push(format!(
                    "group.{}.status.metadata.{}={:?}",
                    port_id,
                    oid,
                    value.as_str()
                ));
            }

            if red {
                result.push(format!("group.{}.status.status=\"red\"", port_id));
            } else if green {
                result.push(format!("group.{}.status.status=\"green\"", port_id));
            }
        }

        result
    }
}

fn calculate_bool(expression: &str, metadata: &HashMap<String, Value>) -> bool {
    match expressions::expression::calculate(expression, metadata) {
        Ok(Ok(value)) => value.as_bool(),
        Err(e) => {
            log::warn!("Failed to parse expression {:?}: {}", expression, e);
            false
        }
        Ok(Err(e)) => {
            log::warn!("Failed to evaluate expression {:?}: {:?}", expression, e);
            false
        }
    }
}
